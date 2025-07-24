use async_trait::async_trait;
use jsonwebtoken::{encode, Header, decode, Validation, TokenData, Algorithm};
use chrono::{Utc, Duration};
use uuid::Uuid;
use crate::entities::token::{Claims, RefreshClaims, TokenType};
use crate::entities::user::User;
use crate::repositories::token::TokenServiceRepository;
use crate::settings::{AppConfig, JwtKeys};
use crate::errors::AuthError;
use crate::{AppState, RedisService};

const JWT_ALGORITHM: Algorithm = Algorithm::HS512;
const ACCESS_DENY_PREFIX: &str = "access_deny";
const REFRESH_DENY_PREFIX: &str = "refresh_deny";


#[derive(Clone)]
pub struct JwtService {
    keys: JwtKeys,
    access_expiration: Duration,
    refresh_expiration: Duration,
}
impl JwtService {
    pub fn new(config: &AppConfig) -> Self {
        JwtService { 
            keys: JwtKeys::from(config), 
            access_expiration: Duration::minutes(config.jwt_expiration_minutes), 
            refresh_expiration: Duration::days(config.refresh_token_exp_days),
        }
    }

    pub fn create_jwt(&self, user: &User) -> Result<String, AuthError> {
        let now = Utc::now();
        let exp = (now + self.access_expiration).timestamp() as usize;

        let claims = Claims {
            sub: user.id.to_string(),
            email: user.email.clone(),
            admin: user.is_admin,
            verified: user.is_verified,
            exp,
            token_type: TokenType::Access,
            iat: now.timestamp() as usize,
        };

        encode(&Header::new(JWT_ALGORITHM), &claims, &self.keys.encoding).map_err(AuthError::from)
    }

    pub fn create_refresh_jwt(&self, user_id: &Uuid) -> Result<String, AuthError> {
        let now = Utc::now();
        let exp = (now + self.refresh_expiration).timestamp() as usize;

        let claims = RefreshClaims {
            sub: user_id.to_string(),
            exp,
            token_type: TokenType::Refresh,
            iat: now.timestamp() as usize,
        };

        encode(&Header::new(JWT_ALGORITHM), &claims, &self.keys.refresh_encoding).map_err(AuthError::from)
    }

    pub fn decode_jwt(&self, token: &str) -> Result<TokenData<Claims>, AuthError> {
        let mut validation = Validation::new(JWT_ALGORITHM);
        validation.validate_exp = true;
        // validation.set_issuer(&["your-issuer"]);
        // validation.set_audience(&["your-audience"]);

        decode::<Claims>(
            token, 
            &self.keys.decoding, 
            &validation
        )
        .map_err(AuthError::from)
    }

    pub fn decode_refresh_jwt(&self, token: &str) -> Result<TokenData<RefreshClaims>, AuthError> {
        let mut validation = Validation::new(JWT_ALGORITHM);
        validation.validate_exp = true;
    
        decode::<RefreshClaims>(
            token,
            &self.keys.refresh_decoding,
            &validation,
        )
        .map_err(AuthError::from)
    }
}


#[async_trait]
impl TokenServiceRepository for JwtService {
    fn create_jwt(&self, user: &User) -> Result<String, AuthError> {
        self.create_jwt(user)
    }

    fn create_refresh_jwt(&self, user_id: &Uuid) -> Result<String, AuthError> {
        self.create_refresh_jwt(user_id)
    }

    fn decode_jwt(&self, token: &str) -> Result<TokenData<Claims>, AuthError> {
        self.decode_jwt(token)
    }

    fn decode_refresh_jwt(&self, token: &str) -> Result<TokenData<RefreshClaims>, AuthError> {
        self.decode_refresh_jwt(token)
    }

    async fn revoke_refresh_token(&self, token: &str, state: &AppState) -> Result<(), AuthError> {
        let claims = self.decode_refresh_jwt(token)?.claims;
        let now = Utc::now().timestamp() as usize;

        if claims.exp <= now {
            return Err(AuthError::InvalidToken);
        }

        let ttl_seconds = (claims.exp - now) as usize;
        state.revoke_token(REFRESH_DENY_PREFIX, token, ttl_seconds).await
    }

     async fn blacklist_access_token(&self, token: &str, state: &AppState) -> Result<(), AuthError> {
        let claims = self.decode_jwt(token)?.claims;
        let now = Utc::now().timestamp() as usize;
        
        if claims.exp <= now {
            return Err(AuthError::InvalidToken);
        }
        let ttl_seconds = (claims.exp - now) as usize;
        state.revoke_token(ACCESS_DENY_PREFIX, token, ttl_seconds).await
    }

    async fn is_revoked(&self, token: &str, state: &AppState) -> Result<bool, AuthError> {
        state.is_token_revoked(ACCESS_DENY_PREFIX, token).await
    }
}

