use chrono::{DateTime, Utc, NaiveDate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Experience {
    pub id: Uuid,
    pub company: String,
    pub role: String,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub description: String,
    pub current_role: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
}