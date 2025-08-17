use actix_web::{http::StatusCode, HttpResponse};
use serde::Serialize;

use crate::errors::{AppError, AuthError};

#[derive(Serialize)]
pub struct JsonError {
    code: u16,
    error: String,
    message: String
}


pub fn json_error(status_code: StatusCode, error: &str, message: &str) -> HttpResponse {
    HttpResponse::build(status_code).json(JsonError {
        code: status_code.as_u16(),
        error: error.to_string(),
        message: message.to_string(),
    })
}

// Helper function to handle AboutHandler errors
pub fn handle_handler_error(e: AppError) -> HttpResponse {
    match e {
        AppError::Conflict(msg) => json_error(
            StatusCode::CONFLICT, 
            "Conflict", 
            &msg
        ),
        AppError::NotFound(msg) => json_error(
            StatusCode::NOT_FOUND, 
            "Not found", 
            &msg
        ),
        AppError::InvalidInput(msg) => json_error(
            StatusCode::BAD_REQUEST, 
            "Invalid input", 
            &msg
        ),
        AppError::UnauthorizedAccess => json_error(
            StatusCode::UNAUTHORIZED, 
            "Unauthorized", 
            "Unauthorized access"
        ),
        AppError::ForbiddenAccess => json_error(
            StatusCode::FORBIDDEN,
            "Forbidden",
            "Forbidden access"
        ),
        AppError::InternalError(_) => json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error",
            "An unexpected error occurred"
        ),
        AppError::ServiceUnavailable(msg) => json_error(
            StatusCode::SERVICE_UNAVAILABLE, 
            "Service unavailable", 
            &msg
        ),
        _ => json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal server error",
            "Something went wrong"
        ),
    }
}

pub fn handle_auth_handler_error(e: AuthError) -> HttpResponse {
    match e {
        AuthError::TokenExpired => return json_error(
            StatusCode::UNAUTHORIZED, 
            "Unauthorized", 
            "Token has expired"
        ),
        AuthError::InvalidToken => return json_error(
            StatusCode::UNAUTHORIZED,
            "Unauthorized",
            "Malformed or invalid token"
        ),
        AuthError::InvalidUserId => return json_error(
            StatusCode::BAD_REQUEST,
            "Bad request",
            "Invalid user ID"
        ),
        AuthError::WrongCredentials => return json_error(
            StatusCode::UNAUTHORIZED,
            "Unauthorized",
            "Wrong credentials"
        ),
        AuthError::RevokedToken => return json_error(
            StatusCode::UNAUTHORIZED,
            "Token Revoked",
            "Token has been revoked"
        ),
        _ => {
            return json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error",
                "Something went wrong"
            )
        }
    }
}