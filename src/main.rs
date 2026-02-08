use tokio::net::TcpListener;
use tokio::signal;
use tracing::info;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use laterfeed::app;
use laterfeed::config::Config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = envy::from_env::<Config>()?;
    let port = config.port;

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            EnvFilter::try_from_default_env()
                .or_else(|_| EnvFilter::try_new("laterfeed=info,tower_http=warn"))?,
        )
        .init();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async {
            let (router, api, pool) = app(config).await;

            info!("generating openapi.json");
            std::fs::write("./openapi.json", api.to_pretty_json().unwrap()).unwrap();

            let listener = TcpListener::bind(format!("0.0.0.0:{port}")).await.unwrap();
            info!("listening on {}", listener.local_addr().unwrap());
            axum::serve(listener, router)
                .with_graceful_shutdown(shutdown_signal())
                .await
                .unwrap();

            info!("shutting down, closing database connection pool");
            pool.close().await;
        });

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
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
