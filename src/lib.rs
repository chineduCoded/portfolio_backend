use redis::Client as RedisClient;

mod domain;
mod interfaces;
mod infrastructure;
pub mod errors;
pub mod settings;
pub mod constants;
pub mod graceful_shutdown;
pub mod background_task;

pub use domain::{entities, use_cases};
pub use interfaces::{handlers, repositories, middlewares};
pub use infrastructure::{auth, db, web};

use auth::jwt::JwtService;
use repositories::sqlx_repo::SqlxRepo;
use use_cases::auth::AuthHandler;

pub struct AppState {
    pub auth_handler: AppAuthHandler,
    pub redis_client: Option<RedisClient>,
}

pub type AppAuthHandler = AuthHandler<SqlxRepo, JwtService>;

impl AppState {
    pub fn new(config: &settings::AppConfig, pool: sqlx::PgPool) -> Self {
        let jwt_service = JwtService::new(config);
        let user_repo = SqlxRepo::new(pool);
        let auth_handler = AuthHandler::new(user_repo, jwt_service);

        let redis_client = config.redis_url.as_ref().and_then(|url| {
            RedisClient::open(url.as_str())
                .map_err(|e| tracing::error!("Redis connection error: {}", e))
                .ok()
        });

        AppState { 
            auth_handler,
            redis_client 
        }
    }
}