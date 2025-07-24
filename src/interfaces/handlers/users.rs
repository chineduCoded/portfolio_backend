use actix_web::{delete, get, http::StatusCode, web, HttpResponse, Responder};
use uuid::Uuid;

use crate::{errors::AppError, handlers::json_error::json_error, repositories::user::UserRepository, use_cases::extractors::AuthClaims, AppState};

#[get("users/me")]
pub async fn me(
    claims: AuthClaims,
    state: web::Data<AppState>
) -> impl Responder {
    let user_id = match Uuid::parse_str(&claims.0.sub) {
        Ok(uuid) => uuid,
        Err(_) => {
            tracing::warn!("Invalid user ID in claims: {}", claims.0.sub);
            return json_error(
            StatusCode::BAD_REQUEST,
            "Bad Request",
            "Invalid user ID in claims"
        )}
    };

    match state.auth_handler.me(user_id).await {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(AppError::NotFound(_)) => {
            tracing::warn!("User not found for ID: {}", user_id);
            json_error(
                StatusCode::NOT_FOUND,
                "Not Found",
                "User not found or does not exist"
            )
        }
        
        Err(e) => {
            tracing::error!("Error fetching user data: {}", e);
            json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error",
                "Something went wrong"
            )
        }
    }
}

#[get("/users/{user_id}")]
pub async fn get_user(
    state: web::Data<AppState>,
    user_id: web::Path<Uuid>,
    claims: AuthClaims,
) -> impl Responder {
    let user_uuid = match Uuid::parse_str(&claims.0.sub) {
        Ok(uuid) => uuid,
        Err(_) => {
            return json_error(
                StatusCode::BAD_REQUEST,
                "Bad Request",
                "Invalid user ID in claims"
            )
        }
    };

    let current_user = match state.auth_handler.user_repo.get_user_by_id(&user_uuid).await {
        Ok(Some(user)) => user,
        Ok(None) => return HttpResponse::NotFound().json("User not found"),
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    };

    match state.auth_handler.get_current_user(user_id.into_inner(), &current_user).await {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(AppError::NotFound(msg)) => {
            tracing::warn!("User not found for ID: {}", user_uuid);
            return json_error(
                StatusCode::NOT_FOUND,
                "Not Found",
                &msg
            )
        }
        Err(e) => {
            tracing::error!("Error fetching user data: {}", e);
            json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error",
                "Something went wrong"
            )
        }
    }
}

#[delete("/users/{user_id}")]
pub async fn delete_user(
    state: web::Data<AppState>,
    user_id: web::Path<Uuid>,
    claims: AuthClaims,
) -> impl Responder {
    let user_uuid = match Uuid::parse_str(&claims.0.sub) {
        Ok(uuid) => uuid,
        Err(_) => {
            return json_error(
                StatusCode::BAD_REQUEST,
                "Bad Request",
                "Invalid user ID in claims"
            )
        }
    };

    let current_user = match state.auth_handler.user_repo.get_user_by_id(&user_uuid).await {
        Ok(Some(user)) => user,
        Ok(None) => return HttpResponse::NotFound().json("User not found"),
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    };

    match state.auth_handler.delete_user(user_id.into_inner(), &current_user).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(AppError::ForbiddenAccess) => {
            return json_error(
                StatusCode::FORBIDDEN,
                "Forbidden",
                "Forbidden access"
            )
        }
        Err(AppError::NotFound(msg)) => {
            return json_error(
                StatusCode::NOT_FOUND,
                "User Not Found",
                &msg
            )
        }
        Err(AppError::Conflict(msg)) => {
            return json_error(
                StatusCode::CONFLICT,
                "Conflicts",
                &msg
            )
        }
        Err(_) => {
            return json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error",
                "Something went wrong"
            )
        }
    }
}