use actix_web::{ 
    middleware::NormalizePath, 
    web, 
    App, HttpServer
};
use portfolio_backend::{
    auth::{jwt::JwtService, password::hash_password}, 
    db::postgres::create_pool, 
    entities::{token::AuthResponse, user::User}, 
    errors::AppError, middlewares::auth::AuthMiddleware, 
    repositories::sqlx_repo::{SqlxAboutMeRepo, SqlxUserRepo}, 
    routes::configure_routes, 
    settings::{AppConfig, AppEnvironment}, 
    use_cases::{about::AboutHandler, auth::AuthHandler}, AppState
};
use redis::AsyncCommands;
use reqwest::Client;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;
use std::{net::TcpListener, sync::Arc, time::Duration};
use async_trait::async_trait;
use portfolio_backend::entities::{user::{LoginUser, NewUser}};

#[derive(Clone)]
pub struct TestApp {
    pub state: Arc<AppState>,
    pub address: String,
    pub db_pool: PgPool,
    pub client: Client,
    pub config: AppConfig,
}

impl TestApp {
    pub async fn spawn() -> Self {
        let config = test_config();

        let db_pool = create_pool(&config.database_url)
            .await
            .expect("Failed to create test DB pool");

        sqlx::query("SELECT 1")
            .execute(&db_pool)
            .await
            .expect("Database connection failed");

        sqlx::query("TRUNCATE TABLE users, user_audit RESTART IDENTITY CASCADE")
            .execute(&db_pool)
            .await
            .expect("Failed to truncate tables");
        

        let redis_pool = config.redis_url.as_ref().and_then(|url| {
            let cfg = deadpool_redis::Config::from_url(url);
            cfg.create_pool(Some(deadpool_redis::Runtime::Tokio1))
                .map_err(|e| tracing::error!("Redis pool creation error: {}", e))
                .ok()
        });

        if let Some(pool) = &redis_pool {
            let mut conn = pool.get().await.expect("Failed to get Redis connection");
            conn.ping::<String>()
                .await
                .expect("Failed to ping Redis");
        }
            
        sqlx::migrate!("./migrations")
            .run(&db_pool)
            .await
            .expect("Failed to run migrations");

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let address = format!("http://127.0.0.1:{}", port);

        let state = Arc::new(AppState {
            auth_handler: AuthHandler::new(
                SqlxUserRepo::new(db_pool.clone()), 
                JwtService::new(&config)
            ),
            about_handler: AboutHandler::new(SqlxAboutMeRepo::new(db_pool.clone())),
            redis_pool,
        });


        let state_clone = state.clone();
        let server = HttpServer::new(move || {
            App::new()
                .app_data(web::Data::from(state_clone.clone()))
                .wrap(NormalizePath::trim())
                .wrap(AuthMiddleware)
                .configure(configure_routes)
        })
        .listen(listener)
        .expect("Failed to bind server")
        .workers(config.worker_count)
        .run();

        tokio::spawn(server);

        let client = Client::new();
        while client.get(&format!("{}/api/v1/admin/health", address)).send().await.is_err() {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        Self { 
            state,
            address,
            db_pool,
            client,
            config,
         }
    }

    pub async fn build_with_transaction(&self) -> Transaction<'static, Postgres> {
        self.db_pool.begin().await.expect("Failed to begin transaction")
    }

    #[allow(dead_code)]
    pub async fn debug_print_users(&self) {
        let users = sqlx::query!("SELECT email, is_verified, is_admin FROM users")
            .fetch_all(&self.db_pool)
            .await
            .unwrap();
        println!("Current users in DB: {:#?}", users);
    }
}

fn test_config() -> AppConfig {
    AppConfig {
        env: AppEnvironment::Testing,
        database_url: "postgres://chinedum:adored89@127.0.0.1:5432/test_db".into(),
        redis_url: Some("redis://localhost:6379".into()),
        jwt_secret: "test_jwt_secret_that_is_long_enough_for_hs512_1234567890".into(),
        refresh_token_secret: "test_refresh_secret_that_is_long_enough_1234567890".into(),
        port: 0,
        cors_allowed_origins: vec!["*".to_string()],
        name: "Portfolio Backend Test".to_string(),
        host: "127.0.0.1".to_string(),
        worker_count: 1,
        jwt_expiration_minutes: 1,
        refresh_token_exp_days: 1,
    }
}

#[async_trait]
pub trait AuthTestHelpers: Send + Sync {
    async fn register_user(&self, user: &NewUser) -> reqwest::Response;
    async fn create_admin_user(&self) -> (NewUser, LoginUser);
    async fn create_regular_user(&self) -> (NewUser, LoginUser);
    async fn insert_user(&self, new_user: &NewUser);
    async fn get_user(&self, email: &str) -> User;
    async fn login_user(&self, credentials: &LoginUser) -> AuthResponse;
    async fn count_users(&self) -> Result<u64, AppError>;
}

#[async_trait]
impl AuthTestHelpers for TestApp {
    async fn register_user(&self, user: &NewUser) -> reqwest::Response {
        self.client
            .post(&format!("{}/api/v1/auth/register", self.address))
            .json(user)
            .send()
            .await
            .expect("Failed to register user")
    }

    async fn create_admin_user(&self) -> (NewUser, LoginUser) {
        let email = format!("admin-{}@example.com", Uuid::new_v4());
        let password = "AdminPass123!";

        let new_user = NewUser {
            email: email.clone(),
            username: Some("adminuser".to_string()),
            password: password.into(),
            is_admin: true,
            is_verified: true,
        };

        self.insert_user(&new_user).await;

        let login_user = LoginUser {
            email,
            password: password.into(),
        };

        (new_user, login_user)
    }

    async fn create_regular_user(&self) -> (NewUser, LoginUser) {
        let email = format!("test-{}@example.com", Uuid::new_v4());
        let password = "ValidPass123!";

        let new_user = NewUser {
            email: email.clone(),
            username: Some("testuser".to_string()),
            password: password.into(),
            is_admin: false,
            is_verified: true,
        };
        

        self.insert_user(&new_user).await;

        let login_user = LoginUser {
            email,
            password: password.into(),
        };

        (new_user, login_user)
    }

    async fn insert_user(&self, new_user: &NewUser) {
        let password_hash = hash_password(&new_user.password).expect("Failed to hash password");
        let user = new_user.prepare_for_insert(password_hash, false);

        sqlx::query!(
            "INSERT INTO users (email, username, password_hash, is_admin, is_verified, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)",
            user.email,
            user.username,
            user.password_hash,
            user.is_admin,
            user.is_verified,
            user.created_at,
            user.updated_at
        )
        .execute(&self.db_pool)
        .await
        .unwrap();
    }

    async fn get_user(&self, email: &str) -> User {
        let user = sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE email = $1",
            email
        )
        .fetch_one(&self.db_pool)
        .await
        .unwrap();

        user
    }

    async fn login_user(&self, credentials: &LoginUser) -> AuthResponse {
        let response = self.client
            .post(&format!("{}/api/v1/auth/login", self.address))
            .json(credentials)
            .send()
            .await
            .expect("Failed to login user");

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            panic!("Login failed: ({}): {}", status, body);
        }

        response.json().await.expect("Failed to parse login response")
    }

    async fn count_users(&self) -> Result<u64, AppError> {
        let count: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM users WHERE deleted_at IS NULL")
            .fetch_one(&self.db_pool)
            .await
            .map_err(AppError::from)?
            .unwrap_or(0);

        Ok(count as u64)
    }
}