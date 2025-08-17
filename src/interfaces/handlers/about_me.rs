use actix_http::StatusCode;
use actix_multipart::{form::MultipartForm, MultipartError};
use actix_web::{
    error::{ContentTypeError, JsonPayloadError}, 
    web, 
    Either, 
    HttpResponse, 
    Responder
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    entities::about_me::{AboutMeUpload, DeleteAboutMeQuery, NewAboutMe}, 
    errors::AppError, use_cases::extractors::AdminClaims, 
    utils::markdown::read_markdown_file, 
    AppState
};



pub async fn create_about_me(
    _claims: AdminClaims,
    state: web::Data<AppState>,
    data_input: Result<Either<MultipartForm<AboutMeUpload>, web::Json<NewAboutMe>>, actix_web::Error>
) -> impl Responder {
    // Handle extractor errors first
    let either = match data_input {
        Ok(either) => either,
        Err(e) => {
            let mut error = serde_json::json!({
                "error": "Invaid request",
                "message": "Error processing input"
            });

            let (status, details) = if let Some(cte) = e.as_error::<ContentTypeError>() {
                (
                    StatusCode::UNSUPPORTED_MEDIA_TYPE,
                    format!("Content type error: {}", cte)
                )
            } else if let Some(mpe) = e.as_error::<MultipartError>() {
                let msg = match mpe {
                    MultipartError::Incomplete => "Incomplete field data",
                    MultipartError::ContentTypeParse => "Could not parse Content-Type header",
                    MultipartError::ContentTypeMissing => "Missing Content-Type header",
                    MultipartError::Parse(_) => "Failed to parse multipart data",
                    MultipartError::Payload(_) => "Payload error",
                    _ => "Invalid multipart form data"
                };
                (
                    StatusCode::BAD_REQUEST,
                    format!("{}: {}", msg, mpe)
                )
            } else if let Some(jpe) = e.as_error::<JsonPayloadError>() {
                match jpe {
                    JsonPayloadError::ContentType => (
                        StatusCode::UNSUPPORTED_MEDIA_TYPE,
                        format!("JSON content type error: {}", jpe)
                    ),
                    JsonPayloadError::Deserialize(err) => (
                        StatusCode::BAD_REQUEST,
                        format!("JSON parsing error: {}", err)
                    ),
                    _ => (
                        StatusCode::BAD_REQUEST,
                        format!("JSON payload error: {}", jpe)
                    )
                }
            } else {
                (
                    StatusCode::BAD_REQUEST,
                    format!("Unknown error: {}", e)
                )
            };


            error["details"] = serde_json::Value::from(details);

            return HttpResponse::build(status).json(error);
        }
    };

    // Process valid input
    match either {
        Either::Left(file_input) => {
            let form = file_input.into_inner();
            
            if let Err(err) = form.metadata.0.validate() {
                return HttpResponse::BadRequest().json(
                    serde_json::json!({"error": "Metadata validation error", "details": err.to_string()})
                );
            }

            let file_name = form.markdown_file.file_name.clone();
            let file_path = form.markdown_file.file.path();
            
            // Read and validate markdown file
            let content = match read_markdown_file(
                file_name.as_deref(), 
                file_path, 2 * 1024 * 1024
            ).await {
                Ok(c) => c,
                Err(e) => {
                    return HttpResponse::BadRequest().json(
                        serde_json::json!({
                            "error": "Markdown file error", 
                            "details": format!("{:?}", e)
                        })
                    );
                }
            };

            // Create NewAboutMe from file content
            let new_about_me = NewAboutMe {
                content_markdown: content,
                effective_date: form.metadata.0.effective_date,
            };
            
            match state.about_handler.create_about_me(new_about_me).await {
                Ok(response) => HttpResponse::Created().json(response),
                Err(e) => handle_handler_error(e),
            }
            
        }
        Either::Right(text_input) => {
            let text_data = text_input.into_inner();
            if let Err(err) = text_data.validate() {
                return HttpResponse::BadRequest().json(
                    serde_json::json!({"error": "Validation error", "details": err.to_string()})
                );
            }
            
            match state.about_handler.create_about_me(text_data).await {
                Ok(response) => HttpResponse::Created().json(response),
                Err(e) => handle_handler_error(e),
            }   
        }
    }
}

pub async fn get_about_me(
    state: web::Data<AppState>
) -> impl Responder {
    match state.about_handler.get_about_me().await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => handle_handler_error(e)
    }
}

pub async fn delete_about_me(
    _claims: AdminClaims,
    path: web::Path<Uuid>,
    query: web::Query<DeleteAboutMeQuery>,
    state: web::Data<AppState>
) -> impl Responder {
    let id = path.into_inner();
    let hard_delete = query.hard_delete.unwrap_or(false);

    match state.about_handler.delete_about_me(id, hard_delete).await {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(e) => handle_handler_error(e)
    }
}

// Helper function to handle AboutHandler errors
fn handle_handler_error(e: AppError) -> HttpResponse {
    match e {
        AppError::Conflict(msg) => HttpResponse::Conflict().json(
            serde_json::json!({"error": "Conflict", "message": msg})
        ),
        AppError::NotFound(msg) => HttpResponse::NotFound().json(
            serde_json::json!({"error": "Not found", "message": msg})
        ),
        AppError::InvalidInput(msg) => HttpResponse::BadRequest().json(
            serde_json::json!({"error": "Bad request", "message": msg})
        ),
        AppError::InternalError(msg) => HttpResponse::InternalServerError().json(
            serde_json::json!({"error": "Internal server error", "message": msg})
        ),
        AppError::ServiceUnavailable(msg) => HttpResponse::ServiceUnavailable().json(
            serde_json::json!({"error": "Service unavailable", "message": msg})
        ),
        _ => HttpResponse::InternalServerError().json(
            serde_json::json!({"error": "Something went wrong"})
        ),
    }
}