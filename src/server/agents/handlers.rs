use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tracing::error;
use uuid::Uuid;

use crate::server::SharedState;

/// ==================== GET /agents ====================
#[derive(Debug, Deserialize, Serialize)]
pub struct ListAgentsElement {
    pub id: Uuid,
    pub name: String,
    pub is_connected: bool,
    pub last_seen: NaiveDateTime,
}

type ListAgentsResponse = Vec<ListAgentsElement>;

pub(crate) async fn list_agents(
    State(state): State<Arc<SharedState>>,
) -> Result<Json<ListAgentsResponse>, StatusCode> {
    let agents = sqlx::query_as!(
        ListAgentsElement,
        "SELECT id, name, is_connected, last_seen FROM agents"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|err| {
        error!(error = ?err, "failed to get agents from database");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(agents))
}

/// ==================== DELETE /agents/{id} ====================
pub(crate) async fn delete_agent(
    State(state): State<Arc<SharedState>>,
    Path(id): Path<Uuid>,
) -> StatusCode {
    let res = sqlx::query!("DELETE FROM agents WHERE id = $1", id)
        .execute(&state.db)
        .await;

    match res {
        Ok(_) => StatusCode::OK,
        Err(err) => {
            error!(error = ?err, "failed to delete agent from database");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
