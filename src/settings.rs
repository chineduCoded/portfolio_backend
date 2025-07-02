use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use dotenv::dotenv;
use jsonwebtoken::{DecodingKey, EncodingKey};
use std::{env, fmt, str::FromStr};
use zeroize::Zeroizing;

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AppEnvironment {
    Development,
    Production,
    Testing,
}

impl FromStr for AppEnvironment {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "development" => Ok(AppEnvironment::Development),
            "production" => Ok(AppEnvironment::Production),
            "testing" => Ok(AppEnvironment::Testing),
            _ => Err(ConfigError::Message(format!("Invalid environment: {}", s))),
        }
    }
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct AppConfig {
    #[serde(default = "default_env")]
    pub env: AppEnvironment,

    #[serde(default = "default_name")]
    pub name: String,

    #[serde(default = "default_port")]
    pub port: u16,

    #[serde(default = "default_host")]
    pub host: String,

    #[serde(default = "default_worker_count")]
    pub worker_count: usize,

    #[serde(default)]
    pub database_url: String,

    #[serde(default)]
    pub redis_url: Option<String>,

    #[serde(default = "default_cors_origins")]
    pub cors_allowed_origins: Vec<String>,

    #[serde(default)]
    pub jwt_secret: String,

    #[serde(default = "default_jwt_expiration")]
    pub jwt_expiration_minutes: i64,

    #[serde(default)]
    pub refresh_token_secret: String,

    #[serde(default = "default_refresh_expiration")]
    pub refresh_token_exp_days: i64,
}

fn default_env() -> AppEnvironment {
    AppEnvironment::Development
}
fn default_name() -> String {
    "Portfolio-API".to_string()
}
fn default_port() -> u16 {
    8080
}
fn default_host() -> String {
    "127.0.0.1".to_string()
}
fn default_worker_count() -> usize {
    num_cpus::get()
}
fn default_cors_origins() -> Vec<String> {
    vec!["*".to_string()]
}
fn default_jwt_expiration() -> i64 {
    15
}
fn default_refresh_expiration() -> i64 {
    7
}

impl AppConfig {
    pub fn new() -> Result<Self, ConfigError> {
        dotenv().ok();

        let raw_env = env::var("APP_ENV").unwrap_or_else(|_| "development".into());
        let env_name = AppEnvironment::from_str(&raw_env)
            .map_err(|_| ConfigError::Message(format!("Invalid APP_ENV value: {}", raw_env)))?;

        let builder = Config::builder()
            .add_source(File::with_name("config/default").required(false))
            .add_source(File::with_name(&format!("config/{}", env_name.to_string().to_lowercase())).required(false))
            .add_source(Environment::with_prefix("APP").separator("_").ignore_empty(true));

        let mut config: Self = builder.build()?.try_deserialize()?;

        config.env = env_name;

        // Inject critical env values if missing
        config.database_url = fill_or_env(config.database_url, "APP_DATABASE_URL")?;
        config.jwt_secret = fill_or_env(config.jwt_secret, "APP_JWT_SECRET")?;
        config.refresh_token_secret = fill_or_env(config.refresh_token_secret, "APP_REFRESH_TOKEN_SECRET")?;

         if config.redis_url.is_none() {
            config.redis_url = env::var("APP_REDIS_URL").ok();
        }
        

        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), ConfigError> {
        let mut errors = Vec::new();

        if self.database_url.trim().is_empty() {
            errors.push("DATABASE_URL cannot be empty");
        }
        if self.jwt_secret.len() < 32 {
            errors.push("JWT_SECRET must be at least 32 characters");
        }
        if self.refresh_token_secret.len() < 32 {
            errors.push("REFRESH_TOKEN_SECRET must be at least 32 characters");
        }
        if self.is_production() && self.cors_origins().iter().any(|o| o == "*") {
            errors.push("Wildcard CORS (*) is not allowed in production");
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ConfigError::Message(errors.join(", ")))
        }
    }

    pub fn is_production(&self) -> bool {
        self.env == AppEnvironment::Production
    }

    pub fn cors_origins(&self) -> Vec<String> {
        self.cors_allowed_origins
            .iter()
            .flat_map(|origin| origin.split(','))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }
}

fn fill_or_env(current: String, env_key: &str) -> Result<String, ConfigError> {
    if current.trim().is_empty() {
        env::var(env_key).map_err(|_| ConfigError::Message(format!("{env_key} must be set")))
    } else {
        Ok(current)
    }
}

impl fmt::Display for AppEnvironment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            AppEnvironment::Development => "development",
            AppEnvironment::Production => "production",
            AppEnvironment::Testing => "testing",
        };
        write!(f, "{s}")
    }
}

trait Redact {
    fn redact(&self) -> &str;
}

impl Redact for str {
    fn redact(&self) -> &str {
        if self.is_empty() {
            "[MISSING]"
        } else if self.len() < 32 {
            "[TOO_SHORT]"
        } else {
            "[REDACTED]"
        }
    }
}

impl Redact for String {
    fn redact(&self) -> &str {
        self.as_str().redact()
    }
}

impl fmt::Debug for AppConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppConfig")
            .field("env", &self.env)
            .field("name", &self.name)
            .field("port", &self.port)
            .field("host", &self.host)
            .field("worker_count", &self.worker_count)
            .field("database_url", &self.database_url.redact())
            .field("cors_allowed_origins", &self.cors_allowed_origins)
            .field("jwt_secret", &self.jwt_secret.redact())
            .field("jwt_expiration_minutes", &self.jwt_expiration_minutes)
            .field("refresh_token_secret", &self.refresh_token_secret.redact())
            .field("refresh_token_exp_days", &self.refresh_token_exp_days)
            .finish()
    }
}

#[derive(Clone)]
pub struct JwtKeys {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
    pub refresh_encoding: EncodingKey,
    pub refresh_decoding: DecodingKey,
}

impl From<&AppConfig> for JwtKeys {
    fn from(config: &AppConfig) -> Self {
        let jwt_secret = Zeroizing::new(config.jwt_secret.clone());
        let refresh_secret = Zeroizing::new(config.refresh_token_secret.clone());

        JwtKeys {
            encoding: EncodingKey::from_secret(jwt_secret.as_bytes()),
            decoding: DecodingKey::from_secret(jwt_secret.as_bytes()),
            refresh_encoding: EncodingKey::from_secret(refresh_secret.as_bytes()),
            refresh_decoding: DecodingKey::from_secret(refresh_secret.as_bytes()),
        }
    }
}

impl fmt::Debug for JwtKeys {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JwtKeys")
            .field("encoding", &"[REDACTED]")
            .field("decoding", &"[REDACTED]")
            .field("refresh_encoding", &"[REDACTED]")
            .field("refresh_decoding", &"[REDACTED]")
            .finish()
    }
}