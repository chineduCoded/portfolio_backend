use actix_web::HttpRequest;
use chrono::Utc;
use uuid::Uuid;
use validator::Validate;

use crate::entities::token::{AuthResponse, TokenType};
use crate::entities::user::{LoginUser, NewUser, NewUserResponse, PublicUser, User};
use crate::errors::{AppError, AuthError};
use crate::interfaces::repositories::user::UserRepository;
use crate::auth::password::{hash_password, verify_password};
use crate::repositories::token::TokenServiceRepository;
use crate::{is_token_invalid, AppState, TokenCheckMode};

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

        let existing_count = self.user_repo.count_users().await?;

        let is_first_user = existing_count == 0;
        if !is_first_user && request.is_admin {
            return Err(AppError::Conflict("Only the first user can be an admin".to_string()));
        }

        let user_insert = request.prepare_for_insert(hashed_password, is_first_user);

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
    pub async fn refresh_token(
        &self, 
        token: &str,
        state: &AppState
    ) -> Result<AuthResponse, AuthError> {
        if token.trim().is_empty() {
            return  Err(AuthError::InvalidToken);
        }

        let decoded = self.token_service.decode_refresh_jwt(token)?;

        if decoded.claims.token_type != TokenType::Refresh {
            return Err(AuthError::InvalidTokenType);
        }

        if decoded.claims.exp < Utc::now().timestamp() as usize {
            return Err(AuthError::TokenExpired);
        }

        let redis_pool = state.redis_pool.as_ref().ok_or(AuthError::TokenCreation)?;
        if is_token_invalid(redis_pool, token, TokenCheckMode::Exists).await? {
            tracing::warn!("Refresh token not found in Redis or has been used: {}", token);
            return Err(AuthError::RevokedToken)
        }

        let user_id = Uuid::parse_str(&decoded.claims.sub)
            .map_err(|_| AuthError::InvalidUserId)?;
        
        let user = self.user_repo.get_user_by_id(&user_id)
            .await
            .map_err(|e| {
                tracing::error!("Database error during refresh: {}", e);
                AuthError::WrongCredentials
            })?
            .ok_or_else(|| {
                tracing::warn!("Refresh attempt for non-existent user: {}", user_id);
                AuthError::WrongCredentials
            })?;

        if user.deleted_at.is_some() {
            tracing::warn!("Refresh attempt for deactivated user: {}", user_id);
            return Err(AuthError::WrongCredentials);
        }
        
        self.create_auth_response(&user)
    }

    pub async fn get_current_user(
        &self, 
        user_id: Uuid, 
        current_user: &User
    ) -> Result<PublicUser, AppError> {
        if current_user.id != user_id && !current_user.is_admin {
            return Err(AppError::ForbiddenAccess);
        }

        self.user_repo.get_user_by_id(&user_id)
            .await?
            .map(PublicUser::from)
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))
    }

    pub async fn delete_user(
        &self,
        user_id: Uuid,
        current_user: &User
    ) -> Result<(), AppError> {
        if current_user.id != user_id && !current_user.is_admin {
            return Err(AppError::ForbiddenAccess);
        }

        self.user_repo.delete_user(&user_id, &current_user.id).await
    }

    pub async fn me(&self, user_id: Uuid) -> Result<PublicUser, AppError> {
        self.user_repo.get_user_by_id(&user_id)
            .await?
            .map(PublicUser::from)
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))
    }

    pub async fn logout(
        &self, 
        refresh_token: &str, 
        access_token: &str,
        state: &AppState
    ) -> Result<(), AuthError> {
        let access_claims = self.token_service.decode_jwt(access_token)?;
        let refresh_claims = self.token_service.decode_refresh_jwt(refresh_token)?;
    
        if access_claims.claims.sub != refresh_claims.claims.sub {
            return Err(AuthError::TokenUserMismatch);
        }

        if access_claims.claims.token_type != TokenType::Access {
            return Err(AuthError::InvalidTokenType);
        }

        if refresh_claims.claims.token_type != TokenType::Refresh {
            return Err(AuthError::InvalidTokenType);
        }

        self.token_service.revoke_refresh_token(refresh_token, state).await?;
        self.token_service.blacklist_access_token(access_token, state).await?;

        Ok(())
    }

    /// Extract token from Authorization header
    pub fn extract_token(&self, request: &HttpRequest) -> Option<String> {
        request.headers()
            .get("Authorization")
            .and_then(|header| header.to_str().ok())
            .and_then(|header| {
                if header.starts_with("Bearer ") {
                    Some(header[7..].to_string())
                } else {
                    None
                }
            })
    }
}