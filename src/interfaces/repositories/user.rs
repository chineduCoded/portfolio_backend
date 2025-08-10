use async_trait::async_trait;
use uuid::Uuid;
use std::borrow::Cow;

use crate::{
    entities::user::{User, UserInsert}, 
    errors::AppError, 
    repositories::sqlx_repo::SqlxUserRepo,
};


#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn check_connection(&self) -> Result<(), AppError>;
    async fn user_exists(&self, id: &Uuid) -> Result<bool, AppError>;
    async fn count_users(&self) -> Result<u64, AppError>;
    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, AppError>;
    async fn create_user(&self, user: &UserInsert) -> Result<Uuid, AppError>;
    async fn get_user_by_id(&self, id: &Uuid) -> Result<Option<User>, AppError>;
    async fn delete_user(&self, id: &Uuid, deleted_by: &Uuid) -> Result<(), AppError>;
    async fn purge_soft_deleted_users(&self) -> Result<u64, AppError>;
}

impl SqlxUserRepo {
    pub fn new(pool: sqlx::PgPool) -> Self {
        SqlxUserRepo { pool }
    }
}

#[async_trait]
impl UserRepository for SqlxUserRepo {
    async fn check_connection(&self) -> Result<(), AppError> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(AppError::from)
    }

    async fn user_exists(&self, id: &Uuid) -> Result<bool, AppError> {
        let exists: Option<bool> = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM users WHERE id = $1 AND deleted_at IS NULL)",
            id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(AppError::from)?;

        let exists = exists.unwrap_or(false);

        Ok(exists)
    }

    async fn count_users(&self) -> Result<u64, AppError> {
        let count: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM users WHERE deleted_at IS NULL")
            .fetch_one(&self.pool)
            .await
            .map_err(AppError::from)?
            .unwrap_or(0);

        Ok(count as u64)
    }

    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE email = $1 AND deleted_at IS NULL",
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
                updated_at,
                deleted_at,
                deleted_by
            ) 
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING id
            "#,
            user.email,
            user.username,
            user.password_hash,
            user.is_admin,
            user.is_verified,
            user.created_at,
            user.updated_at,
            user.deleted_at,
            user.deleted_by
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

    async fn delete_user(&self, id: &Uuid, deleted_by: &Uuid) -> Result<(), AppError> {
        let result = sqlx::query!(
            r#"
            UPDATE users
            SET 
                deleted_at = NOW(),
                deleted_by = $2
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id,
            deleted_by
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            match e {
                sqlx::Error::Database(db_err) if db_err.code() == Some(Cow::Borrowed("23503")) => {
                    AppError::Conflict("User has associated records".to_string())
                }
                _ => AppError::from(e)
            }
        })?;

        if result.rows_affected() == 0 {
            return if self.user_exists(id).await? {
                Err(AppError::Conflict("User is already deleted".to_string()))
            } else {
                Err(AppError::NotFound("User not found".to_string()))
            };
        }

        Ok(())
    }

    async fn purge_soft_deleted_users(&self) -> Result<u64, AppError> {
        let result = sqlx::query!(
            "DELETE FROM users WHERE deleted_at < NOW() - INTERVAL '7 days'"
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}