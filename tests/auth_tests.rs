mod test_utils;
mod test_user;

use std::time::Duration;

// use redis::AsyncCommands;
use serde_json::Value;
use test_utils::*;
// use test_user::*;

use reqwest::StatusCode;
use portfolio_backend::{auth::jwt::JwtService, entities::user::{LoginUser, NewUser}};
use uuid::Uuid;

#[actix_rt::test]
async fn verify_migrations() {
    let app = TestApp::spawn().await;
    let version = sqlx::query!("SELECT version FROM _sqlx_migrations ORDER BY version DESC LIMIT 1")
        .fetch_one(&app.db_pool)
        .await
        .unwrap();
    println!("Latest migration version: {}", version.version);
}

#[actix_rt::test]
async fn register_user_returns_201_for_valid_input() {
    let app = TestApp::spawn().await;
    let transaction = app.build_with_transaction().await;

    println!("Database URL: {}", app.config.database_url);
    println!("JWT Secret: {}", app.config.jwt_secret);
    println!("Redis available: {}", app.state.redis_pool.is_some());

    let unique_email = format!("test-{}@example.com", Uuid::new_v4());
    let user = NewUser {
        email: unique_email,
        username: Some(Uuid::new_v4().to_string()),
        password: "ValidPass123!".into(),
        is_admin: false,
        is_verified: false,
    }; 

    let response = app.register_user(&user).await;

    let status = response.status();
    let body = response.text().await.unwrap();

    if status == StatusCode::INTERNAL_SERVER_ERROR {
        println!("Error body: {}", body);
    }

    assert_eq!(status, StatusCode::CREATED);

    transaction.rollback().await.expect("Failed to rollback transaction");
}

#[actix_rt::test]
async fn login_returns_valid_tokens() {
    let app = TestApp::spawn().await;
    let transaction = app.build_with_transaction().await;

    let (_, user_login) = app.create_regular_user().await;
    let auth = app.login_user(&user_login).await;

    println!("Auth tokens: {:?}", auth);

    assert!(!auth.access_token.is_empty());
    assert!(!auth.refresh_token.is_empty());
    assert_eq!(auth.token_type, "Bearer");

    transaction.rollback().await.expect("Failed to rollback transaction");
}

#[actix_rt::test]
async fn me_endpoint_returns_user_data() {
    let app = TestApp::spawn().await;
    let _transaction = app.build_with_transaction().await;

    let (_, user_login) = app.create_regular_user().await;
    let auth = app.login_user(&user_login).await;

    let response = app.client
        .get(&format!("{}/api/v1/users/me", app.address))
        .bearer_auth(&auth.access_token)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let user: Value = response.json().await.unwrap();
    assert!(user.is_object());
}

#[actix_rt::test]
async fn protected_endpoints_require_auth() {
    let app = TestApp::spawn().await;
    let _transaction = app.build_with_transaction().await;

    let (_, user_login) = app.create_regular_user().await;
    let _auth = app.login_user(&user_login).await;
    
    let response = app.client
        .get(&format!("{}/api/v1/users/me", app.address))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[actix_rt::test]
async fn test_invalid_login() {
    let app = TestApp::spawn().await;
    let _transaction = app.build_with_transaction().await;

    app.register_user(&valid_user()).await;

    let response = app.client
        .post(&format!("{}/api/v1/auth/login", app.address))
        .json(&invalid_login())
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body: Value = response.json().await.unwrap();
    assert_eq!(body["error"], "Wrong credentials");
}

#[actix_rt::test]
async fn test_expired_token() {
    let app = TestApp::spawn().await;
    let transaction = app.build_with_transaction().await;
    
    let (_, user_login) = app.create_regular_user().await;

    let mut config = app.config.clone();
    config.jwt_expiration_minutes = 0;

    let jwt_service = JwtService::new(&config);

    let user = app.get_user(&user_login.email).await;

    let token = jwt_service.create_jwt(&user).expect("Failed to create JWT");

    while app.client.get(&format!("{}/api/v1/admin/health", app.address)).send().await.is_err() {
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    let response = app.client
        .get(&format!("{}/api/v1/users/me", app.address))
        .bearer_auth(&token)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    transaction.rollback().await.expect("Failed to rollback transaction");
}

#[actix_rt::test]
async fn test_admin_access_control() {
    let app = TestApp::spawn().await;
    let _transaction = app.build_with_transaction().await;

    let (new_user, admin_login) = app.create_admin_user().await;
    println!("Created admin user: {:?}", new_user);
    
    let auth = app.login_user(&admin_login).await;
    
    let response = app.client
        .get(&format!("{}/api/v1/admin/dashboard", app.address))
        .bearer_auth(&auth.access_token)
        .send()
        .await
        .unwrap();

    assert!(response.status().is_success());

    let (_, user_login) = app.create_regular_user().await;
    let user_auth = app.login_user(&user_login).await;

    let response = app.client
        .get(&format!("{}/api/v1/admin/dashboard", app.address))
        .bearer_auth(&user_auth.access_token)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body: Value = response.json().await.unwrap();
    assert!(
        body["error"].as_str().unwrap_or_default().contains("Admin access required"),
        "Unexpected error message: {:?}",
        body["error"]
    );
}

fn valid_user() -> NewUser {
    NewUser {
        email: "test@example.com".into(),
        username: Some("testuser".to_string()),
        password: "ValidPass123!".into(),
        is_admin: false,
        is_verified: false,
    }
}

#[allow(dead_code)]
fn valid_login() -> LoginUser {
    LoginUser {
        email: "test@example.com".into(),
        password: "ValidPass123!".into(),
    }
}

fn invalid_login() -> LoginUser {
    LoginUser {
        email: "nonexistent@example.com".into(),
        password: "wrongpassword".into(),
    }
}