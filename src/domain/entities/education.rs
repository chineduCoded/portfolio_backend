use chrono::{DateTime, Utc, NaiveDate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Education {
    pub id: Uuid,
    pub institution: String,
    pub degree: String,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
}