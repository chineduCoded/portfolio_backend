use actix_web::{get, HttpResponse, Responder};
use std::env;

#[get("/")]
pub async fn home() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "message": "Welcome to my Portfolio Web API!",
        "status": "Ok",
        "version": env!("CARGO_PKG_VERSION"),
        "author": "Chinedu Elijah Okoronkwo",
        "repository": "https://github.com/chineduCoded/portfolio_backend.git",
        "documentation": "/docs"
    }))
}