use chrono::{DateTime, Utc, NaiveDate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Project {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub description: String,
    pub content_markdown: String,
    pub technologies: Vec<String>,
    pub featured_image: Option<String>,
    pub github_url: Option<String>,
    pub live_url: Option<String>,
    pub date: NaiveDate,
    pub published: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
}