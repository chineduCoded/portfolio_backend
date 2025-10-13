use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct ContactMeForm {
    #[validate(length(min = 2, max = 100))]
    pub name: String,

    #[validate(email)]
    pub email: String,

    #[validate(length(max = 100))]
    pub subject: Option<String>,

    #[validate(length(min = 5, max = 1000))]
    pub message: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ContactMeMessage {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub subject: Option<String>,
    pub message: String,
    pub created_at: DateTime<Utc>,
}