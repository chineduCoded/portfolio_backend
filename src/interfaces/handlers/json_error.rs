use actix_web::{http::StatusCode, HttpResponse};



pub fn json_error(status: StatusCode, error: &str, details: &str) -> HttpResponse {
    HttpResponse::build(status).json(serde_json::json!({
        "error": error,
        "details": details
    }))
}