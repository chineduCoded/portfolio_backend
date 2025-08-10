use uuid::Uuid;

use crate::errors::AppError;

/// Validates if a string is a valid UUID format
pub fn valid_uuid(id: &str) -> Result<Uuid, AppError> {
    Uuid::parse_str(id).map_err(|_| AppError::InvalidInput("Invalid UUID format".to_string()))
}