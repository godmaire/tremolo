use std::{collections::HashMap, net::SocketAddr, process::ExitCode, sync::Arc};

use axum::{
    Router,
    routing::{any, delete, get, post, put},
};
use clap::Args;
use sqlx::{Pool, Postgres};
use tokio::sync::{RwLock, mpsc::Sender};
use tracing::{Level, error, info};

mod agent;
mod api;

use api::*;

#[derive(Debug, Args)]
pub struct Parameters {
    /// The IPv4 or IPv6 address for Tremolo to listen on
    #[arg(long, default_value = "0.0.0.0:8000", env = "TREMOLO_LISTEN_ADDR")]
    listen: SocketAddr,
    /// Log level for the application
    #[arg(long, default_value_t = Level::INFO, env = "TREMOLO_LOG_LEVEL")]
    log_level: Level,
    /// Path to the Sqlite database
    #[arg(long, env = "TREMOLO_DATABASE_URL")]
    database_url: String,
}

struct SharedState {
    db: Pool<Postgres>,
    agents: RwLock<HashMap<String, Sender<agent::Command>>>,
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
    let db = sqlx::PgPool::connect(&params.database_url)
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
    let state = Arc::new(SharedState {
        db,
        agents: RwLock::new(HashMap::new()),
    });

    let app = Router::new()
        .nest(
            "/api/v1",
            Router::new()
                .nest(
                    "/agents",
                    Router::new()
                        .route("/", get(agents::list_agents))
                        .route("/{id}", delete(agents::delete_agent)),
                )
                .nest(
                    "/apps",
                    Router::new()
                        .route("/", get(apps::list_apps))
                        .route("/create", post(apps::create_app))
                        .route("/{id}", get(apps::get_app))
                        .route("/{id}", put(apps::update_app))
                        .route("/{id}", delete(apps::delete_app)),
                ),
        )
        .route("/healthcheck", get(|| async { "OK" }))
        .nest(
            "/ws",
            Router::new().route("/agent", any(agent::connect_agent)),
        )
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
