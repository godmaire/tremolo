use std::{fmt::Debug, sync::Arc};

use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{self, Message, WebSocket},
    },
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use futures_util::stream::{SplitSink, SplitStream, StreamExt};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::server::SharedState;

static TREMOLO_AUTH_HEADER_KEY: &str = "X-Tremolo-Auth";
static TREMOLO_AGENT_NAME_HEADER_KEY: &str = "X-Tremolo-Agent-Name";

#[derive(Debug, Serialize, Deserialize)]
enum AgentMsg {
    Ack,

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
    headers: HeaderMap,
    State(state): State<Arc<SharedState>>,
) -> Result<impl IntoResponse, StatusCode> {
    let auth_token = headers
        .get(TREMOLO_AUTH_HEADER_KEY)
        .ok_or(StatusCode::FORBIDDEN)?
        .to_str()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let agent_name = headers
        .get(TREMOLO_AGENT_NAME_HEADER_KEY)
        .ok_or(StatusCode::BAD_REQUEST)?
        .to_str()
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .to_string();

    // We attempt to update the token's last_used timestamp, but, if there are no rows
    // affected, we know that the token does not exist and the agent is not authenticated
    //
    // If one row was affected, then we return Some(agent_name)
    let expected_rows_affected = 1;
    let is_authorized = sqlx::query!(
        "UPDATE agents_tokens SET last_used = CURRENT_TIMESTAMP WHERE token = $1",
        auth_token
    )
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .rows_affected()
    .eq(&expected_rows_affected);

    if !is_authorized {
        return Err(StatusCode::FORBIDDEN);
    }

    // Create the agent in the database
    sqlx::query!("INSERT INTO agents (name) VALUES ($1)", agent_name)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Upgrade the websocket!!
    let state = state.clone();
    Ok(ws.on_upgrade(|socket| handle_socket(socket, state, agent_name)))
}

async fn handle_socket(socket: WebSocket, _state: Arc<SharedState>, agent_name: String) {
    info!(name = agent_name, "Connected to agent.");

    let (sender, receiver) = socket.split();

    tokio::spawn(handle_sender(sender));
    tokio::spawn(handle_receiver(receiver));
}

async fn handle_sender(_sender: SplitSink<WebSocket, Message>) {
    todo!()
}

async fn handle_receiver(_receiver: SplitStream<WebSocket>) {
    todo!()
}
