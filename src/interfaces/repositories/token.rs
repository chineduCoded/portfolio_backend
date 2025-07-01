use jsonwebtoken::TokenData;
use uuid::Uuid;
use crate::{entities::{token::{Claims, RefreshClaims}, user::User}, errors::AuthError};



pub trait TokenServiceRepository: Send + Sync {
    /// Creates a new JWT for the user
    fn create_jwt(&self, user: &User) -> Result<String, AuthError>;

    /// Creates a new refresh JWT for the user
    fn create_refresh_jwt(&self, user_id: &Uuid) -> Result<String, AuthError>;

    /// Decodes a JWT and returns the claims
    fn decode_jwt(&self, token: &str) -> Result<TokenData<Claims>, AuthError>;

    /// Decodes a refresh JWT and returns the claims
    fn decode_refresh_jwt(&self, token: &str) -> Result<TokenData<RefreshClaims>, AuthError>;

    /// Checks if a JWT is revoked
    fn is_revoked(&self, token: &str) -> Result<bool, AuthError>;
}