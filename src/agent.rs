use std::process::ExitCode;

use clap::Args;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tracing::{Level, error, info};

#[derive(Debug, Args)]
pub struct Parameters {
    /// Log level for the application
    #[arg(long, default_value_t = Level::INFO, env = "TREMOLO_LOG_LEVEL")]
    log_level: Level,

    /// The URL to your Tremolo orchestrator instance
    #[arg(long, env = "TREMOLO_HOST")]
    host: String,

    /// Name of the Tremolo agent
    #[arg(long, env = "TREMOLO_AGENT_NAME")]
    name: String,

    /// Auth token used for connecting to the Tremolo orchestrator
    #[arg(long, env = "TREMOLO_AGENT_TOKEN")]
    auth_token: String,
}

pub async fn start(params: Parameters) -> ExitCode {
    // Setup tracing subscriber for logging
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_line_number(true)
        .with_file(true)
        .with_max_level(params.log_level)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // Craft the request from the URI then add headers for auth and name
    let mut req = params.host.into_client_request().unwrap();
    req.headers_mut().insert(
        crate::server::agent::TREMOLO_AGENT_NAME_HEADER_KEY,
        params.name.parse().unwrap(),
    );
    req.headers_mut().insert(
        crate::server::agent::TREMOLO_AUTH_HEADER_KEY,
        params.auth_token.parse().unwrap(),
    );

    // Connect to the websocket
    let (mut socket, _res) = match tokio_tungstenite::connect_async(req).await {
        Ok((socket, res)) => (socket, res),
        Err(err) => {
            error!(error = ?err, "failed to connect to orchestrator");
            return ExitCode::FAILURE;
        }
    };

    info!("Connected to orchestrator!");

    // Disconnect cleanly
    socket.close(None).await.unwrap();

    ExitCode::SUCCESS
}
