use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::{Validate, ValidationError};
use actix_multipart::form::{json::Json as MpJson, tempfile::TempFile, MultipartForm};

use crate::utils::markdown::safe_markdown_to_html;

// ───── Database Models ───────────────────────────────────────────────

#[derive(Debug, sqlx::FromRow)]
pub struct AboutMe {
    pub id: Uuid,
    pub revision: i32,
    pub content_markdown: String,
    pub effective_date: NaiveDate,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>
}

#[derive(Debug)]
pub struct AboutMeInsert {
    pub revision: i32,
    pub content_markdown: String,
    pub effective_date: NaiveDate,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ───── API Response Models ──────────────────────────────────────────

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AboutMeResponse {
    pub id: Uuid,
    pub revision: i32,
    pub content_markdown: String,
    pub content_html: String,
    pub effective_date: NaiveDate,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl From<AboutMe> for AboutMeResponse {
    fn from(about_me: AboutMe) -> Self {
        Self {
            id: about_me.id,
            revision: about_me.revision,
            content_markdown: about_me.content_markdown.clone(),
            content_html: safe_markdown_to_html(&about_me.content_markdown),
            effective_date: about_me.effective_date,
            created_at: about_me.created_at,
            updated_at: about_me.updated_at,
            deleted_at: about_me.deleted_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AboutMeCreatedResponse {
    pub id: Uuid,
    pub message: String,
}

// ───── Input & Validation ───────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct NewAboutMe {
    #[validate(length(min = 1, message = "Content cannot be empty"))]
    pub content_markdown: String,

    #[validate(custom(function = "validate_effective_date"))]
    pub effective_date: NaiveDate,
}

#[derive(Debug, MultipartForm)]
pub struct AboutMeUpload {
    #[multipart(rename = "markdown_file", limit = "2MB")]
    pub markdown_file: TempFile,

    #[multipart(rename = "metadata")]
    pub metadata: MpJson<AboutMeMetadata>
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct AboutMeMetadata {
    #[validate(custom(function = "validate_effective_date"))]
    pub effective_date: NaiveDate,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateAboutMeRequest {
    #[validate(length(min = 1, message = "Content cannot be empty"))]
    pub content_markdown: String,
    pub effective_date: NaiveDate,
    pub expected_revision: i32,
}

// ───── Helper Functions ─────────────────────────────────────────────

fn validate_effective_date(date: &NaiveDate) -> Result<(), ValidationError> {
    if *date < NaiveDate::from_ymd_opt(1900, 1, 1).unwrap() {
        let mut err = ValidationError::new("invalid_effective_date");
        err.message = Some("Date must be after January 1, 1900".into());
        return Err(err);
    }
    Ok(())
}

// ───── Insert Preparation Logic ─────────────────────────────────────

impl NewAboutMe {
    pub fn new(content_markdown: String, effective_date: NaiveDate) -> Self {
        Self {
            content_markdown,
            effective_date,
        }
    }

    pub fn prepare_for_insert(&self) -> AboutMeInsert {
        AboutMeInsert {
            revision: 0,
            content_markdown: self.content_markdown.clone(),
            effective_date: self.effective_date,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn prepare_for_update(&self) -> AboutMeInsert {
        self.prepare_for_insert()
    }
}
