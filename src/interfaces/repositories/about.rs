use async_trait::async_trait;
use chrono::NaiveDate;
use uuid::Uuid;

use crate::{
    entities::about_me::{AboutMe, AboutMeInsert, AboutMeResponse}, 
    errors::AppError, repositories::sqlx_repo::SqlxAboutMeRepo,

};

#[async_trait]
pub trait AboutRepository: Send + Sync {
    /// Creates the "About Me" content
    async fn create_about_me(&self, about_insert: &AboutMeInsert) -> Result<Uuid, AppError>;

    /// Retrieves the "About Me" content by id
    async fn get_about_me_by_id(&self, id: Uuid) -> Result<AboutMe, AppError>;

    /// Retrieves the current "About Me" content
    async fn get_current_about_me(&self) -> Result<AboutMeResponse, AppError>;

    /// Updates the "About Me" content
    async fn update_about_me_content(&self, id: Uuid, content: &str) -> Result<AboutMe, AppError>;

    /// Get the latest revision of "About Me" content
    async fn get_latest_revision(&self, effective_date: NaiveDate) -> Result<i32, AppError>;

    /// Soft delete (recommended for most cases)
    async fn soft_delete_about_me(&self, id: Uuid) -> Result<(), AppError>;

    /// Hard delete (for compliance/admin use only)
    async fn hard_delete_about_me(&self, id: Uuid) -> Result<(), AppError>;
}

impl SqlxAboutMeRepo {
    pub fn new(pool: sqlx::PgPool) -> Self {
        SqlxAboutMeRepo { pool }
    }
}

#[async_trait]
impl AboutRepository for SqlxAboutMeRepo {
    async fn create_about_me(&self, about_insert: &AboutMeInsert) -> Result<Uuid, AppError> {
        let id = sqlx::query_scalar!(
            r#"
            INSERT INTO about_me (revision, content_markdown, effective_date) 
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
            about_insert.revision,
            about_insert.content_markdown,
            about_insert.effective_date,
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(id)
    }

    async fn get_about_me_by_id(&self, id: Uuid) -> Result<AboutMe, AppError> {
        let about_me = sqlx::query_as!(
            AboutMe,
            r#"SELECT * FROM about_me WHERE id = $1"#,
            id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(about_me.into())
    }

    async fn get_current_about_me(&self) -> Result<AboutMeResponse, AppError> {
        let about_me = sqlx::query_as!(
            AboutMe,
            r#"SELECT * 
            FROM about_me 
            WHERE effective_date <= CURRENT_DATE 
            ORDER BY effective_date DESC, revision DESC
            LIMIT 1
            "#
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(about_me.into())
    }

    async fn update_about_me_content(&self, id: Uuid, content: &str) -> Result<AboutMe, AppError> {
        let updated = sqlx::query_as!(
            AboutMe,
            r#"
            UPDATE about_me
            SET
                content_markdown = $1,
                updated_at = NOW()
            WHERE id = $2
            RETURNING *
            "#,
            content,
            id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::NotFound("AboutMe entry".into()),
            _ => e.into()
        });

        Ok(updated.map_err(|e| AppError::InternalError(e.to_string()))?)
    }

    async fn get_latest_revision(&self, effective_date: NaiveDate) -> Result<i32, AppError> {
        let revision = sqlx::query_scalar!(
            r#"
            SELECT COALESCE(MAX(revision), -1) + 1
            FROM about_me
            WHERE effective_date = $1
            "#,
            effective_date
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(revision.expect("Failed to fetch latest revision"))
    }

    async fn soft_delete_about_me(&self, id: Uuid) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE about_me
            SET deleted_at = NOW()
            WHERE id = $1 and deleted_at IS NULL
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map(|result| {
            if result.rows_affected() == 0 {
                Err(AppError::NotFound("AboutMe record".into()))
            } else {
                Ok(())
            }
        })?
    }

    async fn hard_delete_about_me(&self, id: Uuid) -> Result<(), AppError> {
        sqlx::query!(
            "DELETE FROM about_me WHERE id = $1",
            id
        )
        .execute(&self.pool)
        .await
        .map(|result| {
            if result.rows_affected() == 0 {
                Err(AppError::NotFound("AboutMe record".into()))
            } else {
                Ok(())
            }
        })?
    }
}