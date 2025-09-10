use std::net::SocketAddr;

use axum::{Router, routing::get};
use clap::Args;
use tracing::{Level, info};

#[derive(Debug, Args)]
pub struct Parameters {
    /// The IPv4 or IPv6 address for Tremolo to listen on
    #[arg(long, default_value = "0.0.0.0:8000", env = "TREMOLO_LISTEN_ADDR")]
    listen: SocketAddr,
    /// Log level for the application
    #[arg(long, default_value_t = Level::INFO)]
    log_level: Level,
}

pub async fn start(params: Parameters) {
    // Setup tracing subscriber for logging
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(params.log_level)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // Initialize Router
    let app = Router::new().route("/healthcheck", get(|| async { "OK" }));

    // Start the application
    info!(address = ?params.listen, "Starting server");
    let listener = tokio::net::TcpListener::bind(params.listen).await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
