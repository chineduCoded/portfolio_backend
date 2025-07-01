use actix_web::{
    error::ResponseError,
    http::{header::ContentType, StatusCode},
    HttpResponse
};
use jsonwebtoken::errors::{ErrorKind, Error as JwtError};
use derive_more::derive::Display;
use validator::ValidationErrors;

#[derive(Debug, Display)]
pub enum AppError {
    #[display("Validation error: {message}")]
    ValidationError { message: String },

    #[display("Resources not found")]
    NotFound,

    #[display("Conflict: {_0}")]
    Conflict(String),

    #[display("Unauthorized access")]
    UnauthorizedAccess,

    #[display("Forbidden access")]
    ForbiddenAccess,

    #[display("Internal server error: {_0}")]
    InternalError(String),
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .json(serde_json::json!({"error": self.to_string()}))
    }
    fn status_code(&self) -> StatusCode {
        match *self {
            AppError::ValidationError { .. } => StatusCode::BAD_REQUEST,
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::UnauthorizedAccess => StatusCode::UNAUTHORIZED,
            AppError::ForbiddenAccess => StatusCode::FORBIDDEN,
            AppError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::InternalError(format!("Database error: {}", err))
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

impl From<ValidationErrors> for AppError {
    fn from(e: ValidationErrors) -> Self {
        let messages = e.field_errors().iter().flat_map(|(field, errors)| {
            errors.iter().map(move |error| {
                format!("{}: {}", field, error.message.as_ref().unwrap_or(&"Invalid value".into()))
            })
        }).collect::<Vec<_>>().join("; ");
        AppError::ValidationError { 
            message: format!("Validation failed: {}", messages) 
        }
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
}


