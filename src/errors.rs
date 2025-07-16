use std::borrow::Cow;
use std::fmt;

use actix_web::{
    error::ResponseError,
    http::{header::ContentType, StatusCode},
    HttpResponse
};
use jsonwebtoken::errors::{ErrorKind, Error as JwtError};
use derive_more::Display;
use serde::Serialize;
use validator::ValidationErrors;

#[derive(Debug)]
pub enum AppError {
    ValidationError(Vec<FieldError>),
    NotFound(String),
    Conflict(String),
    UnauthorizedAccess,
    ForbiddenAccess,
    InternalError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::ValidationError(errors) => {
                let messages = errors.iter()
                    .map(|e| format!("{}:{}", e.field, e.message))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "validation error: {}", messages)
            }
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::Conflict(msg) => write!(f, "Conflict: {}", msg),
            AppError::UnauthorizedAccess => write!(f, "Unauthorized access"),
            AppError::ForbiddenAccess => write!(f, "Forbidden access"),
            AppError::InternalError(msg) => write!(f, "Internal server error: {}", msg)
        }
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let body = match self {
            AppError::ValidationError(errors) => {
                serde_json::json!({
                    "error": "Validation failed",
                    "details": errors
                })
            }
            _ => {
                serde_json::json!({"error": self.to_string()})
            }
        };
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .json(body)
    }
    
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::ValidationError(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::UnauthorizedAccess => StatusCode::UNAUTHORIZED,
            AppError::ForbiddenAccess => StatusCode::FORBIDDEN,
            AppError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<ValidationErrors> for AppError {
    fn from(errors: ValidationErrors) -> Self {
        let field_errors = errors
            .field_errors()
            .iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(|e| FieldError {
                    field: field.to_string(),
                    message: e
                        .message
                        .as_ref()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "Invalid value".to_string()),      
                })
            })
            .collect();

        AppError::ValidationError(field_errors)
    }
}

impl AppError {
    pub fn to_http_response(&self) -> HttpResponse {
        self.error_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::Database(e) if e.code() == Some(Cow::Borrowed("23505")) => {
                AppError::Conflict("Database conflict occurred".into())
            }
            sqlx::Error::Database(e) if e.code() == Some(Cow::Borrowed("23503")) => {
                AppError::Conflict("Foreign key violation".into())
            }
            _ => AppError::InternalError(format!("Database error: {}", err))
        }
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::InternalError(err.to_string())
    }
}

impl From<PasswordError> for AppError {
    fn from(err: PasswordError) -> Self {
        AppError::InternalError(err.to_string())
    }
}

#[derive(Debug, Display)]
pub enum AuthError {
    #[display("Invalid token")]
    InvalidToken,

    #[display("Wrong credentials")]
    WrongCredentials,

    #[display("Token creation error")]
    TokenCreation,

    #[display("Token expired")]
    TokenExpired,

    #[display("Missing credentials")]
    MissingCredentials,

    #[display("Missing JWT service")]
    MissingJwtService,

    #[display("Invalid user ID")]
    InvalidUserId,

    #[display("Password error: {_0}")]
    PasswordError(String),

    #[display("Authentication failed")]
    AuthenticationFailed,

    #[display("Forbidden: {_0}")]
    Forbidden(String),

    #[display("Redis not configured")]
    RedisNotConfigured,

    #[display("Token revoked")]
    TokenRevoked,

    #[display("Redis connection failed: {_0}")]
    RedisConnection(String),
    
    #[display("Redis operation failed: {_0}")]
    RedisOperation(String),
}

impl ResponseError for AuthError {
    fn error_response(&self) -> HttpResponse {
        let error_message = match self {
            AuthError::PasswordError(msg) => format!("Password error: {}", msg),
            AuthError::TokenExpired => "Token has expired".to_string(),
            _ => self.to_string(),
        };
        HttpResponse::build(self.status_code())
            .json(serde_json::json!({"error": error_message}))
    }
    fn status_code(&self) -> StatusCode {
        match *self {
            AuthError::InvalidToken => StatusCode::UNAUTHORIZED,
            AuthError::WrongCredentials => StatusCode::UNAUTHORIZED,
            AuthError::TokenCreation => StatusCode::INTERNAL_SERVER_ERROR,
            AuthError::TokenExpired => StatusCode::UNAUTHORIZED,
            AuthError::MissingCredentials => StatusCode::BAD_REQUEST,
            AuthError::MissingJwtService => StatusCode::INTERNAL_SERVER_ERROR,
            AuthError::InvalidUserId => StatusCode::BAD_REQUEST,
            AuthError::PasswordError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AuthError::AuthenticationFailed => StatusCode::UNAUTHORIZED,
            AuthError::Forbidden(_) => StatusCode::FORBIDDEN,
            AuthError::RedisNotConfigured => StatusCode::BAD_REQUEST,
            AuthError::TokenRevoked => StatusCode::UNAUTHORIZED,
            AuthError::RedisConnection(_) => StatusCode::BAD_REQUEST,
            AuthError::RedisOperation(_) => StatusCode::BAD_REQUEST,
        }
    }
}

impl From<JwtError> for AuthError {
    fn from(e: JwtError) -> Self {
        match e.kind() {
            ErrorKind::ExpiredSignature => AuthError::TokenExpired,
            _ => AuthError::InvalidToken,
        }
    }
}

impl From<argon2::password_hash::Error> for AuthError {
    fn from(_: argon2::password_hash::Error) -> Self {
        AuthError::TokenCreation
    }
}

impl From<PasswordError> for AuthError {
    fn from(err: PasswordError) -> Self {
        AuthError::PasswordError(err.to_string());
        AuthError::AuthenticationFailed
    }
}

impl From<ValidationErrors> for AuthError {
    fn from(_: ValidationErrors) -> Self {
        AuthError::MissingCredentials
    }
}

#[derive(Debug, Display)]
pub enum PasswordError {
    #[display("Invalid password parameters: {_0}")]
    InvalidParameters(String),

    #[display("Password hashing failed: {_0}")]
    HashingError(String),

    #[display("Invalid password hash format: {_0}")]
    InvalidHashFormat(String),

    #[display("Password verification failed: {_0}")]
    VerificationError(String),

    #[display("Password must be at least {_0} characters")]
    TooShort(usize),

    #[display("Password requires uppercase, lowercase, digit, and special character")]
    InsufficientComplexity,

    #[display("Password is extremely weak: {_0}")]
    TooWeak(String),

    #[display("Password is weak: {_0}")]
    Weak(String),
    
    #[display("Unknown password strength score")]
    UnknownStrength,

    #[display("Failed to evaluate password strength")]
    EvaluationFailed,

    #[display("Password is too weak: {_0}")]
    WeakWithFeedback(String)
}

#[derive(Debug, Serialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
}
