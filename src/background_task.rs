use tokio::time::{interval, Duration};

use crate::repositories::{sqlx_repo::SqlxRepo, user::UserRepository};

pub async fn start_purge_task(repo: SqlxRepo) {
    let mut interval = interval(Duration::from_secs(60 * 60 * 24));

    loop {
        interval.tick().await;

        match repo.purge_soft_deleted_users().await {
            Ok(count) => tracing::info!("Purged {} soft-deleted users", count),
            Err(e) => tracing::error!("Purge failed: {}", e)
        }
    }
}