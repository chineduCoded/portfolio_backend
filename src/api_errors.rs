use actix_multipart::MultipartError;
use derive_more::Display;
use actix_web::{HttpResponse, ResponseError};
use serde_json::json;

#[derive(Debug, Display)]
pub enum ApiError {
    #[display("Internal server error")]
    InternalServerError,

    #[display("Not found: {}", _0)]
    NotFound(String),

    #[display("Validation error: {}", _0)]
    ValidationError(String),

    #[display("Unauthorized: {}", _0)]
    Unauthorized(String),

    #[display("Conflict: {}", _0)]
    Conflict(String),

    #[display("Bad request: {}", _0)]
    BadRequest(String),

    #[display("Invalid content type: {}", _0)]
    InvalidContentType(String),

    #[display("Payload too large: {}", _0)]
    PayloadTooLarge(String),

    #[display("Rate limited: {}", _0)]
    RateLimited(String),

    #[display("Database error: {}", _0)]
    DatabaseError(String),

    #[display("IO error: {}", _0)]
    IoError(String),
}

// Implement ResponseError for Actix-Web integration
impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ApiError::InternalServerError => HttpResponse::InternalServerError().json(json!({
                "error": "internal_server_error",
                "message": self.to_string()
            })),
            ApiError::NotFound(_) => HttpResponse::NotFound().json(json!({
                "error": "not_found",
                "message": self.to_string()
            })),
            ApiError::ValidationError(_) => HttpResponse::BadRequest().json(json!({
                "error": "validation_error",
                "message": self.to_string()
            })),
            ApiError::Unauthorized(_) => HttpResponse::Unauthorized().json(json!({
                "error": "unauthorized",
                "message": self.to_string()
            })),
            ApiError::Conflict(_) => HttpResponse::Conflict().json(json!({
                "error": "conflict",
                "message": self.to_string()
            })),
            ApiError::BadRequest(_) => HttpResponse::BadRequest().json(json!({
                "error": "bad_request",
                "message": self.to_string()
            })),
            ApiError::InvalidContentType(_) => HttpResponse::BadRequest().json(json!({
                "error": "invalid_content_type",
                "message": self.to_string()
            })),
            ApiError::PayloadTooLarge(_) => HttpResponse::PayloadTooLarge().json(json!({
                "error": "payload_too_large",
                "message": self.to_string()
            })),
            ApiError::RateLimited(_) => HttpResponse::TooManyRequests().json(json!({
                "error": "rate_limited",
                "message": self.to_string()
            })),
            ApiError::DatabaseError(_) => HttpResponse::InternalServerError().json(json!({
                "error": "database_error",
                "message": self.to_string()
            })),
            ApiError::IoError(_) => HttpResponse::InternalServerError().json(json!({
                "error": "io_error",
                "message": self.to_string()
            })),
        }
    }
}

// Convenient From implementations for common error types
#[derive(Debug, Display)]
pub enum DatabaseError {
    #[display("Constraint violation: {}", _0)]
    ConstraintViolation(String),
    
    #[display("Connection error: {}", _0)]
    ConnectionError(String),
    
    #[display("Query error: {}", _0)]
    QueryError(String),
}

impl From<DatabaseError> for ApiError {
    fn from(err: DatabaseError) -> Self {
        ApiError::DatabaseError(err.to_string())
    }
}

impl From<validator::ValidationErrors> for ApiError {
    fn from(err: validator::ValidationErrors) -> Self {
        ApiError::ValidationError(err.to_string())
    }
}

impl From<std::io::Error> for ApiError {
    fn from(err: std::io::Error) -> Self {
        ApiError::IoError(err.to_string())
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => ApiError::NotFound("Database row not found".to_string()),
            _ => ApiError::DatabaseError(err.to_string()),
        }
    }
}

impl From<actix_multipart::MultipartError> for ApiError {
    fn from(err: actix_multipart::MultipartError) -> Self {
        match err {
            MultipartError::ContentTypeIncompatible => {
                ApiError::InvalidContentType("Content type incompatible".to_string())
            }
            MultipartError::Payload(_) => {
                ApiError::PayloadTooLarge("File too large".to_string())
            }
            _ => ApiError::BadRequest(err.to_string()),
        }
    }
}