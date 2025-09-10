use std::net::SocketAddr;

use axum::{Router, routing::get};
use clap::Args;

#[derive(Debug, Args)]
pub struct Parameters {
    /// The IPv4 or IPv6 address for Tremolo to listen on
    #[arg(long, default_value = "0.0.0.0:8000", env = "TREMOLO_LISTEN_ADDR")]
    listen: SocketAddr,
}

pub async fn start(params: Parameters) {
    let app = Router::new().route("/healthcheck", get(|| async { "OK" }));

    let listener = tokio::net::TcpListener::bind(params.listen).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
