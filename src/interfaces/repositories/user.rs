use async_trait::async_trait;
use uuid::Uuid;
use std::borrow::Cow;

use crate::{entities::user::{User, UserInsert}, errors::AppError, repositories::sqlx_repo::SqlxRepo};


#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn user_exists(&self, email: &str) -> Result<bool, AppError>;
    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, AppError>;
    async fn create_user(&self, user: &UserInsert) -> Result<Uuid, AppError>;
    async fn get_user_by_id(&self, id: &Uuid) -> Result<Option<User>, AppError>;
}

#[async_trait]
impl UserRepository for SqlxRepo {
    async fn user_exists(&self, email: &str) -> Result<bool, AppError> {
        let exists: Option<bool> = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)",
            email
        )
        .fetch_one(&self.pool)
        .await
        .map_err(AppError::from)?;

        let exists = exists.unwrap_or(false);

        Ok(exists)
    }

    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE email = $1",
            email
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::from)?;

        Ok(user)
    }

    async fn create_user(&self, user: &UserInsert) -> Result<Uuid, AppError> {
        let row = sqlx::query!(
            r#"INSERT INTO users (
                email, 
                username,
                password_hash,
                is_admin,
                is_verified,
                created_at, 
                updated_at
            ) 
            VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id
            "#,
            user.email,
            user.username,
            user.password_hash,
            user.is_admin,
            user.is_verified,
            user.created_at,
            user.updated_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            match e {
                sqlx::Error::Database(db_err) if db_err.code() == Some(Cow::Borrowed("23505")) => {
                    AppError::Conflict("User with this email already exists".to_string())
                }
                _ => AppError::from(e),
            }
        })?;

        Ok(row.id)
    }

    async fn get_user_by_id(&self, id: &Uuid) -> Result<Option<User>, AppError> {
        sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", id)
            .fetch_optional(&self.pool)
            .await
            .map_err(AppError::from)
    }
}