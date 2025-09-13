use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::server::SharedState;

/// ==================== GET /agents ====================
#[derive(Debug, Deserialize, Serialize)]
pub struct ListAgentsElement {
    pub id: i64,
    pub name: String,
    pub is_connected: bool,
    pub last_seen: NaiveDateTime,
}

type ListAgentsResponse = Vec<ListAgentsElement>;

pub(crate) async fn list_agents(State(state): State<Arc<SharedState>>) -> Json<ListAgentsResponse> {
    let agents = sqlx::query_as!(
        ListAgentsElement,
        "SELECT id, name, is_connected, last_seen FROM agents"
    )
    .fetch_all(&state.db)
    .await
    .unwrap();

    Json(agents)
}

/// ==================== DELETE /agents/{id} ====================
pub(crate) async fn delete_agent(
    State(state): State<Arc<SharedState>>,
    Path(id): Path<i64>,
) -> StatusCode {
    sqlx::query!("DELETE FROM agents WHERE id = ?", id)
        .execute(&state.db)
        .await
        .unwrap();

    StatusCode::OK
}
