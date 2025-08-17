use tokio::time::{interval, Duration};

use crate::repositories::{sqlx_repo::SqlxUserRepo, user::UserRepository};

pub async fn start_purge_task(
    repo: SqlxUserRepo,
    mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
) {
    let mut interval = interval(Duration::from_secs(60 * 60 * 24));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    // Skip the first tick to avoid immediate execution
    interval.tick().await;

    loop {
        tokio::select! {
            _ = interval.tick() => {
                match repo.purge_soft_deleted_users().await {
                    Ok(count) => tracing::info!("Purged {} soft-deleted users", count),
                    Err(e) => tracing::error!("Purge failed: {}", e),
                }
            }
            _ = shutdown_rx.recv() => {
                tracing::info!("Purge task shutting down gracefully");
                break;
            }
        }
    }
}