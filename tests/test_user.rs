use chrono::{DateTime, Utc};
use portfolio_backend::{auth::password::hash_password, entities::user::User};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct TestUser {
    pub id: Uuid,
    pub email: String,
    pub username: Option<String>,
    pub password_hash: String,
    pub is_admin: bool,
    pub is_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub deleted_by: Option<Uuid>,
}

impl TestUser {
    pub fn new(email: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            email: email.into(),
            username: None,
            password_hash: "$argon2id$v=19$m=19456,t=2,p=1$OEXfGsz8NRc...".into(),
            is_admin: false,
            is_verified: false,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            deleted_by: None,
        }
    }

    #[allow(dead_code)]
    pub fn admin(mut self) -> Self {
        self.is_admin = true;
        self
    }

    #[allow(dead_code)]
    pub fn verified(mut self) -> Self {
        self.is_verified = true;
        self
    }

    #[allow(dead_code)]
    pub fn with_username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }

    #[allow(dead_code)]
    pub fn deleted(mut self, deleted_by: Uuid) -> Self {
        self.deleted_at = Some(Utc::now());
        self.deleted_by = Some(deleted_by);
        self
    }

    #[allow(dead_code)]
    pub fn into_db_user(self) -> User {
        User {
            id: self.id,
            email: self.email,
            username: self.username,
            password_hash: self.password_hash,
            is_admin: self.is_admin,
            is_verified: self.is_verified,
            created_at: self.created_at,
            updated_at: self.updated_at,
            deleted_at: self.deleted_at,
            deleted_by: self.deleted_by,
        }
    }
}

pub async fn create_test_user(pool: &sqlx::PgPool, mut user: TestUser) -> User {
    let password_hash = hash_password("password123").unwrap();
    user.password_hash = password_hash;
    sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (
            id, email, username, password_hash, 
            is_admin, is_verified, created_at, 
            updated_at, deleted_at, deleted_by
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING *
        "#,
        user.id,
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
    .fetch_one(pool)
    .await
    .unwrap()
}