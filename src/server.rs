use std::net::SocketAddr;

use axum::{Router, routing::get};
use clap::Args;
use sqlx::{Pool, Sqlite, SqlitePool};
use tracing::{Level, info};

#[derive(Debug, Args)]
pub struct Parameters {
    /// The IPv4 or IPv6 address for Tremolo to listen on
    #[arg(long, default_value = "0.0.0.0:8000", env = "TREMOLO_LISTEN_ADDR")]
    listen: SocketAddr,
    /// Log level for the application
    #[arg(long, default_value_t = Level::INFO)]
    log_level: Level,
    /// Path to the Sqlite database
    #[arg(
        long,
        default_value = "/opt/tremolo/tremolo.db",
        env = "TREMOLO_DATABASE_URL"
    )]
    database_url: String,
}

#[derive(Clone)]
struct SharedState {
    db: Pool<Sqlite>,
}

pub async fn start(params: Parameters) {
    // Setup tracing subscriber for logging
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(params.log_level)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // Setup database connection
    let db = SqlitePool::connect(&params.database_url).await.unwrap();

    // Initialize Router
    let state = SharedState { db };
    let app = Router::new()
        .route("/healthcheck", get(|| async { "OK" }))
        .with_state(state);

    // Start the application
    info!(address = ?params.listen, "Starting server");
    let listener = tokio::net::TcpListener::bind(params.listen).await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
