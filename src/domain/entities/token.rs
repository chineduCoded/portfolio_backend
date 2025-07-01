use serde::{ Serialize, Deserialize };

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
}

impl AuthResponse {
    pub fn new(access_token: String, refresh_token: String) -> Self {
        AuthResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub admin: bool,
    pub verified: bool,
    pub exp: usize,
    pub iat: usize,
    // iss: Option<String>,
    // aud: Option<String>,
    // nbf: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshClaims {
    pub sub: String,
    pub iat: usize,
    pub exp: usize,
}