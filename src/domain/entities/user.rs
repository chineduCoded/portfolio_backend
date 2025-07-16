use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use validator::Validate;
use uuid::Uuid;

use crate::domain::password::validate_password_strength;


#[derive(Debug, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub username: Option<String>,
    pub password_hash: String,
    pub is_admin: bool,
    pub is_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub deleted_by: Option<Uuid>
}

#[derive(Debug)]
pub struct UserInsert {
    pub email: String,
    pub username: Option<String>,
    pub password_hash: String,
    pub is_admin: bool, 
    pub is_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub deleted_by: Option<Uuid>
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        UserResponse {
            id: user.id,
            email: user.email,
            username: user.username.unwrap_or_else(|| "Anonymous".to_string()),
            created_at: user.created_at,
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct NewUser {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(
        length(min = 8, message = "Must be at least 8 characters"),
        custom(
            function = "validate_password_strength",
            message = "Must include uppercase, number, and symbol"
        )
    )]
    pub password: String,

    #[serde(default = "default_false")]
    pub is_admin: bool,
    
    #[serde(default = "default_false")]
    pub is_verified: bool,
}

/// Returns false, used for serde default.
fn default_false() -> bool {
    false
}

impl NewUser {
    pub fn prepare_for_insert(&self, password_hash: String) -> UserInsert {
        UserInsert {
            email: self.email.clone(),
            username: None,
            password_hash,
            is_admin: false,
            is_verified: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
            deleted_by: None
        }
    }
}


#[derive(Debug, Deserialize, Validate)]
pub struct LoginUser {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    
    #[validate(length(min = 1, message = "Password cannot be empty"))]
    pub password: String,
}


#[derive(Debug, Serialize)]
pub struct NewUserResponse {
    pub id: Uuid,
    pub message: String,
}

#[derive(Serialize)]
pub struct PublicUser {
    pub id: Uuid,
    pub email: String,
    pub username: Option<String>,
    pub is_admin: bool,
    pub is_verified: bool,
    pub created_at: DateTime<Utc>,
}

impl From<User> for PublicUser {
    fn from(user: User) -> Self {
        PublicUser {
            id: user.id,
            email: user.email,
            username: user.username,
            is_admin: user.is_admin,
            is_verified: user.is_verified,
            created_at: user.created_at,
        }
    }
}

#[derive(Deserialize)]
pub struct LogoutRequest {
    pub refresh_token: String,
}
