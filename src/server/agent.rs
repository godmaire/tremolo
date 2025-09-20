use std::{fmt::Debug, sync::Arc};

use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{self, WebSocket},
    },
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{debug, info, warn};

use crate::server::SharedState;

#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
enum Error {
    #[error("invalid message type")]
    InvalidMessage,
}

#[derive(Debug, Serialize, Deserialize)]
enum ClientMessage {
    AuthRequest { name: String, token: String },
    Ack,
}

#[derive(Debug, Serialize, Deserialize)]
enum ServerMessage {
    Ack,
    Error(Error),
}

/// ==================== WS /ws/agent ====================
pub(crate) async fn connect_agent(
    ws: WebSocketUpgrade,
    State(state): State<Arc<SharedState>>,
) -> impl IntoResponse {
    let state = state.clone();
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<SharedState>) {
    let agent_name = match handle_authentication(&state.db, &mut socket).await {
        Some(name) => name,
        None => {
            warn!("Unauthorized agent attempted to connect");
            return;
        }
    };

    info!(name = agent_name, "Connected to agent.");
}

fn encode_message(msg: ServerMessage) -> ws::Message {
    let msg = serde_json::to_string(&msg).expect("failed to serialize ServerMessage");
    ws::Message::text(msg)
}

fn decode_message(msg: Result<ws::Message, axum::Error>) -> Option<ClientMessage> {
    // Flatten all the options and results into options then propegate up the None values
    // before deserializing the message into a message we understand
    let msg = msg.ok()?.into_text().ok()?.to_string();
    debug!(msg = msg, "Websocket message received");
    serde_json::from_str(&msg).ok()
}

async fn handle_authentication(db: &PgPool, socket: &mut WebSocket) -> Option<String> {
    loop {
        let msg = socket.recv().await?;
        let msg = match decode_message(msg) {
            Some(msg) => msg,
            None => {
                socket
                    .send(encode_message(ServerMessage::Error(Error::InvalidMessage)))
                    .await
                    .ok()?;

                continue;
            }
        };

        // If it's not the message we expect, then we retry in the future
        let (name, token) = match msg {
            ClientMessage::AuthRequest { name, token } => (name, token),
            _ => continue,
        };

        // We attempt to update the token's last_used timestamp, but, if there are no rows
        // affected, we know that the token does not exist and the agent is not authenticated
        //
        // If one row was affected, then we return Some(agent_name)
        let expected_rows_affected = 1;
        return sqlx::query!(
            "UPDATE agents_tokens SET last_used = CURRENT_TIMESTAMP WHERE token = $1",
            token
        )
        .execute(db)
        .await
        .ok()?
        .rows_affected()
        .eq(&expected_rows_affected)
        .then_some(name);
    }
}
