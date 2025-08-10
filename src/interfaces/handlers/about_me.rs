use actix_multipart::form::MultipartForm;
use actix_web::{web, Either, HttpResponse, Responder};
use validator::Validate;

use crate::{entities::about_me::{AboutMeUpload, NewAboutMe}, errors::AppError, use_cases::extractors::AdminClaims, utils::markdown::read_markdown_file, AppState};



pub async fn create_about_me(
    _claims: AdminClaims,
    state: web::Data<AppState>,
    data_input: Result<Either<MultipartForm<AboutMeUpload>, web::Json<NewAboutMe>>, actix_web::Error>
) -> impl Responder {
    // Handle extractor errors first
    let either = match data_input {
        Ok(either) => either,
        Err(e) => {
            return HttpResponse::UnsupportedMediaType().json(
                serde_json::json!({
                    "error": "Content type error",
                    "message": "Request must be either application/json or multipart/form-data",
                    "details": e.to_string()
                })
            );
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

            let file_name = form.markdown_file.file_name;
            let file_path = form.markdown_file.file.path();
            
            // Read and validate markdown file
            let content = match read_markdown_file(
                file_name.as_deref(), 
                file_path, 2 * 1024 * 1024
            ).await {
                Ok(content) => content,
                Err(e) => {
                    return HttpResponse::BadRequest().json(
                        serde_json::json!({
                            "error": "Markdown file error", 
                            "details": e.to_string()
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
        _ => HttpResponse::InternalServerError().json(
            serde_json::json!({"error": "Internal server error"})
        ),
    }
}