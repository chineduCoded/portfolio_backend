use jsonwebtoken::{encode, Header, decode, Validation, TokenData, Algorithm};
use chrono::{Utc, Duration};
use uuid::Uuid;
use crate::entities::token::{Claims, RefreshClaims};
use crate::entities::user::User;
use crate::repositories::token::TokenServiceRepository;
use crate::settings::{AppConfig, JwtKeys};
use crate::errors::AuthError;

const JWT_ALGORITHM: Algorithm = Algorithm::HS512;


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
            refresh_expiration: Duration::days(config.refresh_token_exp_days) 
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
    
    pub fn is_revoked(&self, _token: &str) -> Result<bool, AuthError> {
        // Implement your logic to check if the token is revoked
        // This could involve checking a database or cache
        // For now, we will return false indicating the token is not revoked
        Ok(false)
    }
    
}

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

    fn is_revoked(&self, token: &str) -> Result<bool, AuthError> {
        self.is_revoked(token)
    }
}