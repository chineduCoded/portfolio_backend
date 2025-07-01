use uuid::Uuid;
use validator::Validate;

use crate::entities::token::AuthResponse;
use crate::entities::user::{LoginUser, NewUser, NewUserResponse, User};
use crate::errors::{AppError, AuthError};
use crate::interfaces::repositories::user::UserRepository;
use crate::auth::password::{hash_password, verify_password};
use crate::repositories::token::TokenServiceRepository;

pub struct AuthHandler<R, T>
where 
    R: UserRepository,
    T: TokenServiceRepository,
{
    pub user_repo: R,
    pub token_service: T,
}

impl<R, T> AuthHandler<R, T>
where 
    R: UserRepository,
    T: TokenServiceRepository,
{
    pub fn new(user_repo: R, token_service: T) -> Self {
        AuthHandler { 
            user_repo, 
            token_service 
        }
    }

    /// Registers a new user after validation and password hashing
    pub async fn register(&self, request: NewUser) -> Result<NewUserResponse, AppError> {
        request.validate()?;

        let hashed_password = hash_password(&request.password)?;
        let user_insert = request.prepare_for_insert(hashed_password);

        match self.user_repo.create_user(&user_insert).await {
            Ok(user_d) => Ok(NewUserResponse {
                id: user_d,
                message: "User created successfully".to_string(),
            }),
            Err(AppError::Conflict(msg)) => Err(AppError::Conflict(msg)),
            Err(e) => Err(e),
        }
    }

    /// Logs in a user by validating credentials and generating JWTs
    pub async fn login(&self, request: LoginUser) -> Result<AuthResponse, AuthError> {
        request.validate()?;

        let user = self.user_repo.get_user_by_email(&request.email)
            .await
            .map_err(|_e| AuthError::WrongCredentials)?
            .ok_or_else(|| AuthError::WrongCredentials)?;


        let is_password_valid = verify_password(&request.password, &user.password_hash)
            .map_err(|_| {
                AuthError::WrongCredentials
            })?;
        if !is_password_valid {
            return Err(AuthError::WrongCredentials);
        }

        let response = self.create_auth_response(&user)?;

        tracing::info!("User logged in successfully");
        Ok(response)
    }

    /// Create auth response
    pub fn create_auth_response(&self, user: &User) -> Result<AuthResponse, AuthError> {
        let access_token = self.token_service.create_jwt(user)
            .map_err(|e| {
                tracing::warn!("Failed to create JWT: {}", e);
                AuthError::TokenCreation
            })?;
            
        let refresh_token = self.token_service.create_refresh_jwt(&user.id)
            .map_err(|e| {
                tracing::warn!("Failed to create refresh JWT: {}", e);
                AuthError::TokenCreation
            })?;
        Ok(AuthResponse::new(access_token, refresh_token))
    }

    /// Refreshes the access token using the refresh token
    pub async fn refresh_token(&self, token: &str) -> Result<AuthResponse, AuthError> {
        let decoded = self.token_service.decode_refresh_jwt(token)?;
        let user_id = Uuid::parse_str(&decoded.claims.sub)
            .map_err(|_| AuthError::InvalidUserId)?;
        
        let user = self.user_repo.get_user_by_id(&user_id)
            .await
            .map_err(|_| AuthError::WrongCredentials)?
            .ok_or(AuthError::WrongCredentials)?;
        
        self.create_auth_response(&user)
    }
}