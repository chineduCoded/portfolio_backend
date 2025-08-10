use chrono::{DateTime, Utc, NaiveDate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "skill_category", rename_all = "lowercase")]
pub enum SkillCategory {
    Language,
    Framewwork,
    Tool,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Skill {
    pub id: Uuid,
    pub name: String,
    pub category: SkillCategory,
    #[serde(validate = "validate_proficiency")]
    pub proficiency: i16,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn validate_proficiency(p: i16) -> Result<(), String> {
    if (1..=100).contains(&p) {
        Ok(())
    } else {
        Err("Proficiency must be between 1 and 100".into())
    }
}