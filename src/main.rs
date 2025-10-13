use std::env;

use actix_web::{middleware::NormalizePath, web, App, HttpServer};
use tracing_actix_web::TracingLogger;
use tracing_subscriber::{fmt, EnvFilter};
use portfolio_backend::{
    background_task::start_purge_task, 
    db::postgres::create_pool, 
    graceful_shutdown::shutdown_signal, 
    middlewares::auth::AuthMiddleware, 
    routes::configure_routes, 
    settings::AppConfig, 
    AppState
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,actix_web=info,portfolio_backend=debug"));

    fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .compact()
        .init();

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
            .wrap(TracingLogger::default())
            .wrap(NormalizePath::trim())
            .wrap(AuthMiddleware)
            .configure(configure_routes)
    })
    .bind(server_addr)?
    .run();

    // Create a broadcast channel for shutdown signal
    let (shutdown_sender, shutdown_receiver) = tokio::sync::broadcast::channel(1);

    // Keep an Actix server handle to trigger a graceful stop
    let server_handle = server.handle();

    // Keep a JoinHandle so we can await the task on shutdown
    let purge_handle = tokio::spawn(start_purge_task(
        app_state_clone.auth_handler.user_repo.clone(),
        shutdown_receiver,
    ));

    let res = tokio::select! {
        res = server => res,
        _ = shutdown_signal() => {
            let _ = shutdown_sender.send(());
            server_handle.stop(true).await;
            Ok(())
        },
    };

    let _ = purge_handle.await;

    res
}
