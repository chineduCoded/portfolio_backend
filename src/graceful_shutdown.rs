use tokio::signal;
use tracing::warn;

pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to listen for Ctrl+C");
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{signal, SignalKind};
        let mut sigterm = signal(SignalKind::terminate())
            .expect("Failed to listen for SIGTERM");
        sigterm.recv().await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            warn!("🛑 Ctrl+C received, initiating shutdown...")
        },
        _ = terminate => {
            warn!("🛑 SIGTERM received, initiating shutdown...");
        }
    }
}