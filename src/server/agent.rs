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
use tracing::{info, warn};

use crate::server::SharedState;

#[derive(Debug, Serialize, Deserialize)]
enum AgentMsg {
    Ack,
    AuthRequest { name: String, token: String },

    Invalid,
}

impl From<AgentMsg> for ws::Message {
    fn from(value: AgentMsg) -> Self {
        let msg = serde_json::to_string(&value).expect("failed to serialize AgentMsg");
        ws::Message::text(msg)
    }
}

impl From<ws::Message> for AgentMsg {
    fn from(value: ws::Message) -> Self {
        let msg = match value.into_text() {
            Ok(msg) => msg.to_string(),
            Err(_) => return Self::Invalid,
        };

        serde_json::from_str(&msg).unwrap_or(Self::Invalid)
    }
}

#[derive(Debug, Serialize, Deserialize)]
enum ServerMsg {
    Ack,

    Invalid,
}

impl From<ServerMsg> for ws::Message {
    fn from(value: ServerMsg) -> Self {
        let msg = serde_json::to_string(&value).expect("failed to serialize ServerMsg");
        ws::Message::text(msg)
    }
}

impl From<ws::Message> for ServerMsg {
    fn from(value: ws::Message) -> Self {
        let msg = match value.into_text() {
            Ok(msg) => msg.to_string(),
            Err(_) => return Self::Invalid,
        };

        serde_json::from_str(&msg).unwrap_or(Self::Invalid)
    }
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

            // NOTE: We don't actually care at this point because we're closing the
            // connection anyway
            let _ = socket.send(ServerMsg::Invalid.into()).await;
            return;
        }
    };

    info!(name = agent_name, "Connected to agent.");
}

async fn handle_authentication(db: &PgPool, socket: &mut WebSocket) -> Option<String> {
    // If it's not the message we expect, then they fail
    let msg = socket.recv().await?.ok()?.into();
    let (name, token) = match msg {
        AgentMsg::AuthRequest { name, token } => (name, token),
        _ => return None,
    };

    // We attempt to update the token's last_used timestamp, but, if there are no rows
    // affected, we know that the token does not exist and the agent is not authenticated
    //
    // If one row was affected, then we return Some(agent_name)
    let expected_rows_affected = 1;
    sqlx::query!(
        "UPDATE agents_tokens SET last_used = CURRENT_TIMESTAMP WHERE token = $1",
        token
    )
    .execute(db)
    .await
    .ok()?
    .rows_affected()
    .eq(&expected_rows_affected)
    .then_some(name)
}
