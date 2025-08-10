use chrono::{DateTime, Utc, NaiveDate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use serde_json::Value as JsonValue;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Resume {
    pub id: Uuid,
    pub user_id: Uuid,  // Foreign key to User
    pub version: String, // e.g., "v1.2.0"
    pub is_public: bool,
    pub theme: ResumeTheme, // Enum
    pub metadata: JsonValue, // Stores customizations
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
}

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "resume_theme", rename_all = "kebab-case")]
pub enum ResumeTheme {
    Modern,
    Classic,
    Minimalist,
    Executive
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ResumeHeader {
    pub id: Uuid,
    pub resume_id: Uuid,
    pub full_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub location: Option<String>,
    pub website_url: Option<String>,
    pub github_url: Option<String>,
    pub linkedin_url: Option<String>,
    pub headline: String,  // "Senior Backend Engineer"
    pub summary: String,   // 2-3 sentence bio
    pub display_order: i16
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ResumeExperience {
    pub id: Uuid,
    pub resume_id: Uuid,
    pub experience_id: Uuid, // FK to main Experience table
    pub display_order: i16,
    pub is_highlighted: bool,
    pub custom_description: Option<String> // Override for resume-specific text
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ResumeSkillGroup {
    pub id: Uuid,
    pub resume_id: Uuid,
    pub name: String, // "Languages", "DevOps", etc.
    pub display_order: i16,
    pub skills: Vec<ResumeSkillItem>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResumeSkillItem {
    pub skill_id: Uuid,  // FK to main Skill table
    pub custom_name: Option<String>,
    pub proficiency: Option<i16>, // Override if needed
    pub display_order: i16
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ResumeEducation {
    pub id: Uuid,
    pub resume_id: Uuid,
    pub education_id: Uuid, // FK to main Education table
    pub display_order: i16,
    pub include_description: bool
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ResumeCustomSection {
    pub id: Uuid,
    pub resume_id: Uuid,
    pub title: String, // "Publications", "Certifications"
    pub items: Vec<ResumeCustomItem>,
    pub display_order: i16
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResumeCustomItem {
    pub id: Uuid,
    pub title: String,
    pub subtitle: Option<String>, // e.g., "AWS Certified Developer"
    pub date: Option<NaiveDate>,
    pub description: Option<String>,
    pub url: Option<String>
}