use actix_web::{delete, web, HttpResponse, Responder};
use uuid::Uuid;

use crate::{errors::AppError, repositories::user::UserRepository, use_cases::extractors::AuthClaims, AppState};



#[delete("/users/{user_id}")]
pub async fn delete_user(
    state: web::Data<AppState>,
    user_id: web::Path<Uuid>,
    claims: AuthClaims,
) -> impl Responder {
    let user_uuid = match Uuid::parse_str(&claims.0.sub) {
        Ok(uuid) => uuid,
        Err(_) => return HttpResponse::BadRequest().json("Invalid user ID in claims"),
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
        Err(AppError::ForbiddenAccess) => HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Forbidden"
        })),
        Err(AppError::NotFound(msg)) => HttpResponse::NotFound().json(serde_json::json!({
            "error": msg
        })),
        Err(AppError::Conflict(msg)) => HttpResponse::Conflict().json(serde_json::json!({
            "error": msg
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}