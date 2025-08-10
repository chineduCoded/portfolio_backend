use async_trait::async_trait;
use deadpool_redis::{
    Pool as RedisPool, 
    Runtime,
};
use redis::AsyncCommands;

mod domain;
mod interfaces;
mod infrastructure;
pub mod api_errors;
pub mod errors;
pub mod settings;
pub mod constants;
pub mod graceful_shutdown;
pub mod background_task;
pub mod shared_repos;

pub use domain::{entities, use_cases};
pub use interfaces::{handlers, repositories, middlewares, routes};
pub use infrastructure::{auth, db, utils};

use auth::jwt::JwtService;
use use_cases::auth::AuthHandler;

use crate::{domain::use_cases::about::AboutHandler, errors::AuthError, interfaces::repositories::sqlx_repo::{SqlxAboutMeRepo, SqlxUserRepo}, shared_repos::SharedRepositories};

#[async_trait]
pub trait RedisService {
    async fn revoke_token(&self, prefix: &str, token: &str, ttl: usize) -> Result<(), AuthError>;
    async fn is_token_revoked(&self, prefix: &str, token: &str) -> Result<bool, AuthError>;
}

pub struct AppState {
    pub auth_handler: AppAuthHandler,
    pub about_handler: AboutHandler<SqlxAboutMeRepo>,
    pub redis_pool: Option<RedisPool>,
}

pub type AppAuthHandler = AuthHandler<SqlxUserRepo, JwtService>;

impl AppState {
    pub fn new(
        config: &settings::AppConfig, 
        pool: sqlx::PgPool
    ) -> Self {
        let shared_repos = SharedRepositories::new(pool);
        let jwt_service = JwtService::new(config);

        let auth_handler = AuthHandler::new(shared_repos.user_repo, jwt_service);
        let about_handler = AboutHandler::new(shared_repos.about_repo);
        
        let redis_pool = config.redis_url.as_ref().and_then(|url| {
            let cfg = deadpool_redis::Config::from_url(url);
            cfg.create_pool(Some(Runtime::Tokio1))
                .map_err(|e| {
                    tracing::error!("Redis pool creation error: {}", e)
                })
                .ok()
        });

        AppState { 
            auth_handler,
            about_handler,
            redis_pool 
        }
    }

    /// Helper method to access Redis
    pub async fn with_redis<F, Fut, T>(&self, op: F) -> Result<T, AuthError>
    where 
        F: FnOnce(deadpool_redis::Connection) -> Fut,
        Fut: Future<Output = Result<T, AuthError>>,
    {
        if let Some(pool) = &self.redis_pool {
            let conn = pool.get().await
                .map_err(|e| AuthError::RedisConnection(e.to_string()))?;
            op(conn).await
        } else {
            Err(AuthError::RedisNotConfigured)
        }
    }

    pub async fn check_redis_health(&self) -> &'static str {
        if let Some(pool) = &self.redis_pool {
            match pool.get().await {
                Ok(mut conn) => {
                    match conn.ping::<String>().await {
                        Ok(pong) if pong == "PONG" => "OK".into(),
                        Ok(_) => "Unexpected response",
                        Err(_) => "Ping failed"
                    }
                }
                Err(_) => "Connection failed",
            }
        } else {
            "Not configured".into()
        }
    }
}

#[async_trait]
impl RedisService for AppState {
    async fn revoke_token(&self, prefix: &str, token: &str, ttl_seconds: usize) -> Result<(), AuthError> {
        if ttl_seconds == 0 {
            return Ok(());
        }
        
        self.with_redis(|mut conn| async move {
            conn.set_ex::<_, _, ()>(
                format!("{}:{}", prefix, token),
                "1",
                ttl_seconds as u64
            ).await
            .map_err(|e| AuthError::RedisOperation(e.to_string()))?;
        
            Ok(())
        }).await
    }

    async fn is_token_revoked(&self, prefix: &str, token: &str) -> Result<bool, AuthError> {
        self.with_redis(|mut conn| async move {
            let exists: bool = conn
                .exists(format!("{}:{}", prefix, token))
                .await
                .map_err(|e| AuthError::RedisOperation(e.to_string()))?;
            Ok(exists)
        }).await
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TokenCheckMode {
    Exists,     // Token is invalid if key exists (blacklist)
    NotExists,  // Token is invalid if key doesn't exist (revocation)
}

pub async fn is_token_invalid(
    redis_pool: &RedisPool,
    key: &str,
    mode: TokenCheckMode,
) -> Result<bool, AuthError> {
    let mut conn = redis_pool.get().await
        .map_err(|e| AuthError::RedisOperation(e.to_string()))?;

    let exists = conn.exists(key).await
        .map_err(|e| AuthError::RedisOperation(e.to_string()))?;

    Ok(match mode {
        TokenCheckMode::Exists => exists,        // blacklisted if exists
        TokenCheckMode::NotExists => !exists,    // revoked if not exists
    })
}