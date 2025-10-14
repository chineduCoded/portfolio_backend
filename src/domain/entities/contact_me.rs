use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::{Validate, ValidationErrors};

// ============================== ContactMe Entities ==============================

#[derive(Debug, Deserialize, Validate)]
pub struct NewContactMeForm {
    #[validate(length(min = 2, max = 100))]
    pub name: String,

    #[validate(email)]
    pub email: String,

    #[validate(length(max = 100))]
    pub subject: Option<String>,

    #[validate(length(min = 5, max = 1000))]
    pub message: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ContactMeQuery {
    #[validate(range(min = 1, max = 100))]
    pub limit: Option<u32>,

    #[validate(range(min = 0))]
    pub offset: Option<u32>,
}

#[derive(Debug, Validate)]
pub struct ContactMeFormInsert {
    #[validate(length(min = 2, max = 100))]
    pub name: String,

    #[validate(email)]
    pub email: String,

    #[validate(length(max = 100))]
    pub subject: Option<String>,

    #[validate(length(min = 5, max = 1000))]
    pub message: String,

    pub created_at: DateTime<Utc>,


}

// ============================= DB Models ==============================

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ContactMeMessage {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub subject: Option<String>,
    pub message: String,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// ======================= Responses =======================

#[derive(Debug, Serialize)]
pub struct ContactMeResponse {
    pub message: String,
    pub id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct ContactMeListResponse {
    pub messages: Vec<ContactMeMessage>,
    pub total: i64,
}

// ======================= Conversions =======================

impl TryFrom<NewContactMeForm> for ContactMeFormInsert {
    type Error = ValidationErrors;

    fn try_from(form: NewContactMeForm) -> Result<Self, Self::Error> {
        form.validate()?;

        Ok(Self {
            name: form.name,
            email: form.email,
            subject: form.subject,
            message: form.message,
            created_at: Utc::now(),
        })
    }
}