mod test_utils;
mod test_user;

use serde_json::Value;
use test_utils::*;
use test_user::*;

use reqwest::StatusCode;
use portfolio_backend::{auth::jwt::JwtService, entities::user::{LoginUser, NewUser}};
use uuid::Uuid;

#[actix_rt::test]
async fn register_user_returns_201_for_valid_input() {
    let app = TestApp::spawn().await;

    let response = app.register_user(&valid_user()).await;

    assert_eq!(response.status(), StatusCode::CREATED);
    app.cleanup_test_db().await;
}

#[actix_rt::test]
async fn login_returns_valid_tokens() {
    let app = TestApp::spawn().await;
    app.register_user(&valid_user()).await;

    let auth = app.login_user(&valid_login()).await;

    assert!(!auth.access_token.is_empty());
    assert!(!auth.refresh_token.is_empty());
    assert_eq!(auth.token_type, "bearer");
    app.cleanup_test_db().await;
}

#[actix_rt::test]
async fn me_endpoint_returns_user_data() {
    let app = TestApp::spawn().await;
    app.register_user(&valid_user()).await;
    let auth = app.login_user(&valid_login()).await;

    let response = app.client
        .get(&format!("{}/api/v1/users/me", app.address))
        .bearer_auth(&auth.access_token)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let user: Value = response.json().await.unwrap();
    assert_eq!(user["email"], "test@example.com");
    app.cleanup_test_db().await;
}
#[actix_rt::test]
async fn protected_endpoints_require_auth() {
    let app = TestApp::spawn().await;
    
    let response = app.client
        .get(&format!("{}/api/v1/users/me", app.address))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    app.cleanup_test_db().await;
}

#[actix_rt::test]
async fn test_invalid_login() {
    let app = TestApp::spawn().await;
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

    app.cleanup_test_db().await;
}

#[actix_rt::test]
async fn test_expired_token() {
    let app = TestApp::spawn().await;
    
    let email = format!("test-{}@example.com", Uuid::new_v4());
    let user = create_test_user(&app.db_pool, TestUser::new(email)).await;

    let mut config = app.config.clone();
    config.jwt_expiration_minutes = 0;

    let jwt_service = JwtService::new(&config);

    let token = jwt_service.create_jwt(&user).expect("Failed to create JWT");

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let response = app.client
        .get(&format!("{}/api/v1/users/me", app.address))
        .bearer_auth(&token)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    app.cleanup_test_db().await;
}

#[actix_rt::test]
async fn test_admin_access_control() {
    let app = TestApp::spawn().await;
    app.register_user(&valid_user()).await;
    
    let auth = app.login_user(&valid_login()).await;
    
    let response = app.client
        .get(&format!("{}/api/v1/admin/dashboard", app.address))
        .bearer_auth(&auth.access_token)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    
    app.cleanup_test_db().await;
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