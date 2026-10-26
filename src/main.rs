use std::net::SocketAddr;
use qinping_air_monitor::{create_app, AppConfig, AppState};
use tokio::signal;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::from_env();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("qinping_air_monitor={},tower_http=debug,info", config.log_level).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;

    info!("Starting qinping-air-monitor on {}", addr);
    info!("Cache TTL: {}s", config.cache_ttl_secs);
    if !config.app_key.is_empty() {
        info!("Qingping App Key configured: {}***", &config.app_key[..config.app_key.len().min(4)]);
    } else {
        tracing::warn!("QINGPING_APP_KEY is not set!");
    }

    let state = AppState::new(config);
    let app = create_app(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Listening on {}", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Server shutdown cleanly");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C, shutting down gracefully");
        },
        _ = terminate => {
            info!("Received SIGTERM, shutting down gracefully");
        },
    }
}
