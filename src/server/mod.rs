use std::{net::SocketAddr, process::ExitCode, str::FromStr, sync::Arc};

use axum::{
    Router,
    routing::{delete, get, post, put},
};
use clap::Args;
use sqlx::{
    Pool, Sqlite,
    sqlite::{SqliteConnectOptions, SqliteJournalMode},
};
use tracing::{Level, error, info};

mod agents;
mod app;

use agents::handlers::*;
use app::handlers::*;

#[derive(Debug, Args)]
pub struct Parameters {
    /// The IPv4 or IPv6 address for Tremolo to listen on
    #[arg(long, default_value = "0.0.0.0:8000", env = "TREMOLO_LISTEN_ADDR")]
    listen: SocketAddr,
    /// Log level for the application
    #[arg(long, default_value_t = Level::INFO, env = "TREMOLO_LOG_LEVEL")]
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

pub async fn start(params: Parameters) -> ExitCode {
    // Setup tracing subscriber for logging
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_line_number(true)
        .with_file(true)
        .with_max_level(params.log_level)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // Setup database connection
    let opts = SqliteConnectOptions::from_str(&params.database_url)
        .expect("valid sqlite url")
        .journal_mode(SqliteJournalMode::Wal)
        .create_if_missing(true);
    let db = sqlx::SqlitePool::connect_with(opts)
        .await
        .expect("failed to open database");

    info!("Running database migrations...");
    let res = sqlx::migrate!("src/server/migrations").run(&db).await;
    if let Err(err) = res {
        error!(error = ?err, "Failed to run database migrations.");
        return ExitCode::FAILURE;
    }
    info!("Successfully  ran database migrations.");

    // Initialize Router
    let state = Arc::new(SharedState { db });
    let app = Router::new()
        .nest(
            "/agents",
            Router::new()
                .route("/", get(list_agents))
                .route("/{id}", delete(delete_agent)),
        )
        .nest(
            "/apps",
            Router::new()
                .route("/", get(list_apps))
                .route("/create", post(create_app))
                .route("/{id}", get(get_app))
                .route("/{id}", put(update_app))
                .route("/{id}", delete(delete_app)),
        )
        .route("/healthcheck", get(|| async { "OK" }))
        .with_state(state);

    // Start the application
    info!(address = ?params.listen, "Starting server");
    let listener = match tokio::net::TcpListener::bind(params.listen).await {
        Ok(listener) => listener,
        Err(err) => {
            error!(error = ?err, "Failed to bind listener.");
            return ExitCode::FAILURE;
        }
    };

    axum::serve(listener, app)
        .await
        .expect("this will never error");

    ExitCode::SUCCESS
}
