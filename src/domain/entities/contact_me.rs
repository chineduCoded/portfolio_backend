use chrono::{DateTime, Utc, NaiveDate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct ContactMessage {
  pub id: Uuid,
  pub name: String,
  pub email: String,
  pub message: String,
  pub ip_address: Option<std::net::Ipv4Addr>,
  pub user_agent: Option<String>,
  pub status: i16,
  pub responded_at: Option<chrono::DateTime<chrono::Utc>>,
  pub created_at: chrono::DateTime<chrono::Utc>,
  pub updated_at: chrono::DateTime<chrono::Utc>,
}