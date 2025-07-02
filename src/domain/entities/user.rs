use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use validator::Validate;
use uuid::Uuid;


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
    #[validate(email(message = "Invalid email format"), length(max = 255))]
    pub email: String,

    #[validate(length(min = 8, max = 72, message = "Password must be between 8 and 72 characters"))]
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
        }
    }
    pub fn validate_password_complexity(&self) -> Result<(), validator::ValidationError> {
        let mut has_upper = false;
        let mut has_lower = false;
        let mut has_digit = false;
        let mut has_special = false;

        let password = &self.password;
    
        for ch in password.chars() {
            match ch {
                ch if ch.is_uppercase() => has_upper = true,
                ch if ch.is_lowercase() => has_lower = true,
                ch if ch.is_digit(10) => has_digit = true,
                ch if "!@#$%^&*()-_=+[]{}|;:'\",.<>?/`~".contains(ch) => has_special = true,
                _ => {},
            }
    
            if has_upper && has_lower && has_digit && has_special {
                break;
            }
        }
    
        if has_upper && has_lower && has_digit && has_special {
            Ok(())
        } else {
            Err(validator::ValidationError::new("Password must contain at least one uppercase letter, one lowercase letter, one digit, and one special character."))
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
