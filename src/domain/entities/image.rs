use chrono::{DateTime, Utc, NaiveDate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ImageAsset {
    pub id: Uuid,
    pub user_id: Uuid, // Owner (admin who uploaded)
    pub storage_key: String, // S3 path or filesystem path
    pub public_url: String, // CDN URL
    pub file_name: String,
    pub mime_type: String, // "image/png"
    pub file_size: i32, // KB
    pub dimensions: (i32, i32), // (width, height)
    pub alt_text: Option<String>, // For accessibility
    pub is_public: bool, // Allow public access?
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>> // Soft delete
}

// For upload requests
#[derive(Debug, Deserialize, Validate)]
pub struct ImageUploadRequest {
    #[validate(length(max = 5_242_880))] // 5MB max
    pub file_data: Vec<u8>,
    #[validate(length(min = 1, max = 255))]
    pub file_name: String,
    #[validate(length(max = 125))]
    pub alt_text: Option<String>
}

// If using Cloudinary/Imgix
pub struct ImageReference {
    pub provider_id: String, // "cloudinary:abc123"
    pub transformations: String // "w_500,h_500,c_fill"
}