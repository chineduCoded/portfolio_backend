use actix_web::{
    web,
    http::StatusCode,
    ResponseError,
    HttpResponse,
    error::JsonPayloadError,
};
use serde_json::json;


pub fn config_routes(cfg: &mut web::ServiceConfig) {
    cfg.app_data(web::JsonConfig::default().error_handler(|err, _req| {
        JsonError::from(err).into()
    }));
}

#[derive(Debug)]
pub struct JsonError {
    message: String,
    status: StatusCode
}

// Implement Display for JsonError
impl std::fmt::Display for JsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl ResponseError for JsonError {
    fn status_code(&self) -> StatusCode {
        self.status
    }

    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(self.status).json(json!({ "error": self.message }))
    }
}
impl From<serde_json::Error> for JsonError {
    fn from(err: serde_json::Error) -> Self {
        JsonError {
            message: format!("JSON error: {}", err),
            status: StatusCode::BAD_REQUEST,
        }
    }
}

impl From<JsonPayloadError> for JsonError {
    fn from(err: JsonPayloadError) -> Self {
        JsonError {
            message: format!("JSON payload error: {}", err),
            status: StatusCode::BAD_REQUEST,
        }
    }
}  
