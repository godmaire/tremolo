use std::{fmt::Debug, sync::Arc};

use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use futures_util::{
    SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{self, Receiver};
use tracing::{error, info};

use crate::server::SharedState;

pub static TREMOLO_AUTH_HEADER_KEY: &str = "X-Tremolo-Auth";
pub static TREMOLO_AGENT_NAME_HEADER_KEY: &str = "X-Tremolo-Agent-Name";

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum Response {
    Invalid,
}

impl From<Response> for Message {
    fn from(value: Response) -> Self {
        let msg = serde_json::to_string(&value).expect("failed to serialize AgentMsg");
        Message::text(msg)
    }
}

impl From<Message> for Response {
    fn from(value: Message) -> Self {
        let msg = match value.into_text() {
            Ok(msg) => msg.to_string(),
            Err(_) => return Self::Invalid,
        };

        serde_json::from_str(&msg).unwrap_or(Self::Invalid)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum Command {
    Invalid,
}

impl From<Command> for Message {
    fn from(value: Command) -> Self {
        let msg = serde_json::to_string(&value).expect("failed to serialize ServerMsg");
        Message::text(msg)
    }
}

impl From<Message> for Command {
    fn from(value: Message) -> Self {
        let msg = match value.into_text() {
            Ok(msg) => msg.to_string(),
            Err(_) => return Self::Invalid,
        };

        serde_json::from_str(&msg).unwrap_or(Self::Invalid)
    }
}

/// ==================== WS /ws/agent ====================
pub(super) async fn connect_agent(
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
    sqlx::query!(
        "INSERT INTO agents (name) VALUES ($1) ON CONFLICT DO NOTHING",
        agent_name
    )
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Websocket!!!
    let state = state.clone();
    Ok(ws.on_upgrade(async move |socket| {
        info!(name = agent_name, "Connected to agent.");

        let (tx, rx) = mpsc::channel::<Command>(8);
        state.agents.write().await.insert(agent_name.clone(), tx);

        let (sender, receiver) = socket.split();
        tokio::spawn(handle_sender(sender, rx));
        tokio::spawn(handle_receiver(receiver, state, agent_name));
    }))
}

// Forward messages from the receiver to the agent
async fn handle_sender(mut sender: SplitSink<WebSocket, Message>, mut rx: Receiver<Command>) {
    while let Some(msg) = rx.recv().await {
        match sender.send(msg.into()).await {
            Ok(_) => {}
            Err(err) => {
                error!(error = ?err, "failed to send message to agent")
            }
        }
    }
}

// Handle messages received from the agent
async fn handle_receiver(
    mut receiver: SplitStream<WebSocket>,
    state: Arc<SharedState>,
    agent_name: String,
) {
    while let Some(msg) = receiver.next().await {
        let msg: Response = match msg {
            Ok(msg @ Message::Text(_)) => msg.into(),

            Ok(Message::Close(_)) => {
                let res = sqlx::query!(
                    "UPDATE agents SET is_connected = FALSE, last_seen = CURRENT_TIMESTAMP WHERE name = $1",
                    agent_name
                ).execute(&state.db)
                .await;

                if let Err(err) = res {
                    error!(error = ?err, "failed to update agent in database");
                }

                break;
            }

            // NOOPs
            Ok(Message::Ping(_)) | Ok(Message::Pong(_)) => continue,
            Ok(Message::Binary(_)) => {
                error!("binary format unsupported");
                continue;
            }

            // Error :(
            Err(err) => {
                error!(error = ?err, "failed to get message from agent");
                continue;
            }
        };

        match msg {
            Response::Invalid => todo!(),
        }
    }

    info!("Agent disconnected!!!");
}
