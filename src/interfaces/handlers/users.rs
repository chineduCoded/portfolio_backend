use actix_web::{http::StatusCode, web, HttpResponse, Responder};
use uuid::Uuid;

use crate::{ 
    handlers::json_error::{handle_handler_error, json_error}, 
    repositories::user::UserRepository, 
    use_cases::extractors::AuthClaims, 
    AppState
};

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
        Err(e) => {
            tracing::warn!("User not found for ID: {}", user_id);
            handle_handler_error(e)
        }
    }
}

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
        Err(e) => return handle_handler_error(e)
    };

    match state.auth_handler.get_current_user(user_id.into_inner(), &current_user).await {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(e) => {
            tracing::warn!("User not found for ID: {}", user_uuid);
            handle_handler_error(e)
        }
    }
}

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
        Err(e) => return handle_handler_error(e)
    };

    match state.auth_handler.delete_user(user_id.into_inner(), &current_user).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => handle_handler_error(e)
    }
}