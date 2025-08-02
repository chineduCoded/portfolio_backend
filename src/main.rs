use std::env;

use actix_web::{middleware::NormalizePath, web, App, HttpServer};
use portfolio_backend::{
    background_task::start_purge_task, 
    db::postgres::create_pool, 
    graceful_shutdown::shutdown_signal, 
    middlewares::auth::AuthMiddleware, 
    routes::configure_routes, 
    settings::AppConfig, 
    AppState,
};

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
            .wrap(AuthMiddleware)
            .configure(configure_routes)
    })
    .bind(server_addr)?
    .run();

    tokio::spawn(start_purge_task(app_state_clone.auth_handler.user_repo.clone()));

    tokio::select! {
        res = server => res,
        _ = shutdown_signal() => Ok(()),
    }
}
