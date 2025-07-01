use sqlx::postgres::{PgPool, PgPoolOptions};
use tracing::info;
use std::time::Duration;

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let max_retries = 5;
    let mut retry_count = 0;
    let mut wait_seconds = 2;

    loop {
        match PgPoolOptions::new()
            .max_connections(20)
            .connect(database_url)
            .await
        {
            Ok(pool) => {
                info!("Database connection established.");
                return Ok(pool);
            }
            Err(e) if retry_count < max_retries => {
                retry_count += 1;
                info!(
                    "Failed to connect to database (attempt {}/{}): {}. Retrying in {}s...",
                    retry_count, max_retries, e, wait_seconds);

                tokio::time::sleep(Duration::from_secs(wait_seconds)).await;

                wait_seconds *= 2; // Exponential backoff
            }
            Err(e) => return Err(e),
        }
    }
}