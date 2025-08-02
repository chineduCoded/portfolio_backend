use actix_web::{ 
    middleware::NormalizePath, 
    web, 
    App, HttpServer
};
use portfolio_backend::{
    auth::jwt::JwtService, db::postgres::create_pool, entities::token::AuthResponse, middlewares::auth::AuthMiddleware, routes::configure_routes, settings::{AppConfig, AppEnvironment}, AppState
};
use reqwest::Client;
use sqlx::{PgPool, Postgres, Transaction};
use std::{net::TcpListener, sync::Arc, time::Duration};
use async_trait::async_trait;
use portfolio_backend::entities::{user::{LoginUser, NewUser}};

#[derive(Clone)]
pub struct TestApp {
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
        
        sqlx::migrate!("./migrations")
            .run(&db_pool)
            .await
            .expect("Failed to run migrations");

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let address = format!("http://127.0.0.1:{}", port);

        let mut app_state = AppState::new(&config, db_pool.clone());
        app_state.auth_handler.token_service = JwtService::new(&config);
        let state = Arc::new(app_state);

        let server = HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(state.clone()))
                .wrap(NormalizePath::trim())
                .wrap(AuthMiddleware)
                .configure(configure_routes)
        })
        .listen(listener)
        .expect("Failed to bind server")
        .workers(config.worker_count)
        .run();

        tokio::spawn(server);
        tokio::time::sleep(Duration::from_millis(50)).await;

        Self { 
            address,
            db_pool,
            client: Client::new(),
            config,
         }
    }

    #[allow(dead_code)]
    pub async fn build_with_transaction(&self) -> Transaction<'static, Postgres> {
        self.db_pool.begin().await.expect("Failed to begin transaction")
    }

    pub async fn cleanup_test_db(&self) {
        let tables = vec!["users", "user_audit"];

        for table in tables {
            sqlx::query(&format!("TRUNCATE TABLE {} CASCADE", table))
                .execute(&self.db_pool)
                .await
                .unwrap_or_else(|e| panic!("Failed to truncate table {}: {}", table, e));
        }
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
    async fn login_user(&self, credentials: &LoginUser) -> AuthResponse;
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
}