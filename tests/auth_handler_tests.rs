// use std::sync::Arc;
// use chrono::{Utc, Duration};
// use uuid::Uuid;
// use mockall::{mock, predicate::*};
// use tokio;

// use crate::auth::AuthHandler;
// use crate::auth::jwt::{JwtService, AuthResponse};
// use crate::auth::password::{hash_password};
// use crate::entities::user::{NewUser, LoginUser, User};
// use crate::errors::{AppError, AuthError};
// use crate::settings::{AppConfig, JwtKeys};

// // === Mock Trait for UserRepository ===
// mock! {
//     pub UserRepo {}

//     #[async_trait::async_trait]
//     impl crate::interfaces::repositories::user::UserRepository for UserRepo {
//         async fn user_exists(&self, email: &str) -> Result<bool, AppError>;
//         async fn create_user(&self, new_user: &crate::entities::user::InsertableUser) -> Result<Uuid, AppError>;
//         async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, AppError>;
//     }
// }

// // === Test Config Helper ===
// fn test_config() -> AppConfig {
//     AppConfig {
//         jwt_secret: "super_test_secret_key_for_jwt_signing".to_string(),
//         jwt_expiration_minutes: 60,
//         refresh_token_exp_days: 7,
//         ..Default::default()
//     }
// }

// // === Shared Setup ===
// fn setup_handler() -> (MockUserRepo, JwtService) {
//     let repo = MockUserRepo::new();
//     let jwt = JwtService::new(&test_config());
//     (repo, jwt)
// }

// // === TESTS ===

// #[tokio::test]
// async fn test_register_success() {
//     let (mut repo, jwt) = setup_handler();
//     let email = "new@test.com";

//     repo.expect_user_exists()
//         .with(eq(email))
//         .returning(|_| Ok(false));

//     repo.expect_create_user()
//         .returning(|_| Ok(Uuid::new_v4()));

//     let handler = AuthHandler::new(repo, jwt);

//     let new_user = NewUser {
//         email: email.to_string(),
//         password: "Secret123!".to_string(),
//         password_confirm: "Secret123!".to_string(),
//     };

//     let result = handler.register(new_user).await;
//     assert!(result.is_ok());
// }

// #[tokio::test]
// async fn test_register_email_conflict() {
//     let (mut repo, jwt) = setup_handler();

//     repo.expect_user_exists()
//         .returning(|_| Ok(true));

//     let handler = AuthHandler::new(repo, jwt);

//     let new_user = NewUser {
//         email: "exists@test.com".to_string(),
//         password: "Secret123!".to_string(),
//         password_confirm: "Secret123!".to_string(),
//     };

//     let result = handler.register(new_user).await;
//     assert!(matches!(result, Err(AppError::Conflict(_))));
// }

// #[tokio::test]
// async fn test_login_valid_password() {
//     let (mut repo, jwt) = setup_handler();
//     let email = "valid@test.com";
//     let password = "StrongP@ssw0rd";
//     let hash = hash_password(password).unwrap();

//     repo.expect_get_user_by_email()
//         .with(eq(email))
//         .returning(move |_| {
//             let user = User {
//                 id: Uuid::new_v4(),
//                 email: email.to_string(),
//                 password_hash: hash.clone(),
//             };
//             Ok(Some(user))
//         });

//     let handler = AuthHandler::new(repo, jwt);

//     let result = handler.login(LoginUser {
//         email: email.to_string(),
//         password: password.to_string(),
//     }).await;

//     assert!(result.is_ok());
//     let tokens = result.unwrap();
//     assert!(!tokens.access_token.is_empty());
//     assert!(!tokens.refresh_token.is_empty());
// }

// #[tokio::test]
// async fn test_login_invalid_password() {
//     let (mut repo, jwt) = setup_handler();
//     let email = "invalid@test.com";
//     let correct_hash = hash_password("CorrectP@ss").unwrap();

//     repo.expect_get_user_by_email()
//         .with(eq(email))
//         .returning(move |_| {
//             Ok(Some(User {
//                 id: Uuid::new_v4(),
//                 email: email.to_string(),
//                 password_hash: correct_hash.clone(),
//             }))
//         });

//     let handler = AuthHandler::new(repo, jwt);

//     let result = handler.login(LoginUser {
//         email: email.to_string(),
//         password: "WrongP@ss".to_string(),
//     }).await;

//     assert!(matches!(result, Err(AuthError::PasswordError(_))));
// }

// #[tokio::test]
// async fn test_refresh_token_valid() {
//     let (mut repo, jwt) = setup_handler();
//     let email = "refresh@test.com";
//     let user_id = Uuid::new_v4();

//     repo.expect_get_user_by_email()
//         .with(eq(email))
//         .returning(move |_| {
//             Ok(Some(User {
//                 id: user_id,
//                 email: email.to_string(),
//                 password_hash: "".to_string(),
//             }))
//         });

//     let handler = AuthHandler::new(repo, jwt.clone());

//     let refresh_token = jwt.create_refresh_jwt(&user_id).unwrap();

//     let result = handler.refresh_token(&refresh_token, email).await;

//     assert!(result.is_ok());
//     let resp = result.unwrap();
//     assert!(!resp.access_token.is_empty());
//     assert!(!resp.refresh_token.is_empty());
// }

// #[tokio::test]
// async fn test_jwt_creation_failure() {
//     let mut repo = MockUserRepo::new();
//     let mut jwt = JwtService::new(&test_config());

//     let user_id = Uuid::new_v4();
//     let email = "fail@test.com";
//     let hash = hash_password("password").unwrap();

//     // Inject invalid keys to force JWT failure
//     jwt = JwtService {
//         keys: JwtKeys {
//             encoding: &b"invalid"[..].into(),
//             decoding: &b"invalid"[..].into(),
//         },
//         access_expiration: Duration::minutes(60),
//         refresh_expiration: Duration::days(7),
//     };

//     repo.expect_get_user_by_email()
//         .with(eq(email))
//         .returning(move |_| {
//             Ok(Some(User {
//                 id: user_id,
//                 email: email.to_string(),
//                 password_hash: hash.clone(),
//             }))
//         });

//     let handler = AuthHandler::new(repo, jwt);

//     let result = handler.login(LoginUser {
//         email: email.to_string(),
//         password: "password".to_string(),
//     }).await;

//     assert!(matches!(result, Err(AuthError::TokenCreation)));
// }
