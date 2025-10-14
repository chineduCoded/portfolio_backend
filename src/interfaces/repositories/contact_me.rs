use async_trait::async_trait;
use uuid::Uuid;

use crate::{entities::contact_me::{ContactMeFormInsert, ContactMeMessage}, errors::AppError, repositories::sqlx_repo::SqlxContactMeRepo};

#[async_trait]
pub trait ContactMeRepository: Send + Sync {
    async fn create_contact_message(&self, msg: &ContactMeFormInsert) -> Result<Uuid, AppError>;
    async fn get_contact_message_by_id(&self, id: &Uuid) -> Result<ContactMeMessage, AppError>;
    async fn list_contact_messages(&self) -> Result<Vec<ContactMeMessage>, AppError>;
    async fn count_contact_messages(&self) -> Result<i64, AppError>;
    async fn soft_delete_contact_message(&self, id: &Uuid) -> Result<(), AppError>;
    async fn hard_delete_contact_message(&self, id: &Uuid) -> Result<(), AppError>;
}

impl SqlxContactMeRepo {
    pub fn new(pool: sqlx::PgPool) -> Self {
        SqlxContactMeRepo { pool }
    }
}

#[async_trait]
impl ContactMeRepository for SqlxContactMeRepo {
    async fn create_contact_message(&self, msg: &ContactMeFormInsert) -> Result<Uuid, AppError> {
        let id = sqlx::query_scalar!(
            r#"
            INSERT INTO contact_me_messages (name, email, subject, message) 
            VALUES ($1, $2, $3, $4)
            RETURNING id
            "#,
            msg.name,
            msg.email,
            msg.subject,
            msg.message,
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(id)
    }

    async fn get_contact_message_by_id(&self, id: &Uuid) -> Result<ContactMeMessage, AppError> {
        let contact_msg = sqlx::query_as!(
            ContactMeMessage,
            r#"SELECT * FROM contact_me_messages WHERE id = $1 AND deleted_at IS NULL"#,
            id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(contact_msg.into())
    }

    async fn list_contact_messages(&self) -> Result<Vec<ContactMeMessage>, AppError> {
        let messages = sqlx::query_as!(
            ContactMeMessage,
            r#"SELECT * FROM contact_me_messages WHERE deleted_at IS NULL ORDER BY created_at DESC"#
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(ContactMeMessage::from)
        .collect();

        Ok(messages)
    }

    async fn count_contact_messages(&self) -> Result<i64, AppError> {
        let count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) as "count!" FROM contact_me_messages WHERE deleted_at IS NULL"#
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    async fn soft_delete_contact_message(&self, id: &Uuid) -> Result<(), AppError> {
        sqlx::query!(
            r#"UPDATE contact_me_messages SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL"#,
            id
        )
        .execute(&self.pool)
        .await
        .map(|result| {
            if result.rows_affected() == 0 {
                Err(AppError::NotFound("Contact me not found".into()))
            } else {
                Ok(())
            }
        })?
    }
    async fn hard_delete_contact_message(&self, id: &Uuid) -> Result<(), AppError> {
        sqlx::query!(
            r#"DELETE FROM contact_me_messages WHERE id = $1"#,
            id
        )
        .execute(&self.pool)
        .await
        .map(|result| {
            if result.rows_affected() == 0 {
                Err(AppError::NotFound("Contact me not found".into()))
            } else {
                Ok(())
            }
        })?
    }
}