use std::env;

use actix_web::{get, middleware::NormalizePath, web, App, HttpResponse, HttpServer, Responder};
use portfolio_backend::{
    db::postgres::create_pool, 
    graceful_shutdown::shutdown_signal,
    background_task::start_purge_task,
    handlers::{auth::{login, register}, 
    system::health_check, users::delete_user}, 
    middlewares::auth::AuthMiddleware, 
    settings::AppConfig, AppState
};

#[get("/")]
async fn home() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "message": "Welcome to my Portfolio Web API!",
        "status": "operational",
        "version": env!("CARGO_PKG_VERSION"),
        "author": "Chinedu Elijah Okoronkwo",
        "repository": "https://github.com/chineduCoded/portfolio_api.git",
        "documentation": "/docs"
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();

    let config = match AppConfig::new() {
        Ok(cfg) => {
            tracing::info!("Loaded configuration: {:?}", cfg);
            cfg
        },
        Err(e) => {
            tracing::error!("Configuration error: {}", e);
            std::process::exit(1);
        }
    };

    let pool = create_pool(&config.database_url)
        .await
        .expect("Failed to create database connection pool");

    let app_state = web::Data::new(
        AppState::new(&config, pool.clone())
    );

    let server_addr = format!("{}:{}", config.host, config.port);
    
    tracing::info!(
        "🚀 Starting Portfolio API v{} on {}",
        env!("CARGO_PKG_VERSION"),
        server_addr
    );
    
    let app_state_clone = app_state.clone();

    let server = HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(NormalizePath::trim())
            .service(home)

            .service(
                web::scope("/auth")
                    .service(register)
                    .service(login)
            )

            .service(
                web::scope("/api")
                    .wrap(AuthMiddleware)
                    .service(delete_user)
                    .service(health_check)
            )
    })
    .bind(server_addr)?
    .run();

    tokio::spawn(start_purge_task(app_state_clone.auth_handler.user_repo.clone()));

    tokio::select! {
        res = server => res,
        _ = shutdown_signal() => Ok(()),
    }
}
