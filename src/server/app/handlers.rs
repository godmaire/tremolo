use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::server::SharedState;

/// ==================== GET /apps ====================
#[derive(Debug, Deserialize, Serialize)]
pub struct ListAppsElement {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
}

pub type ListAppsResponse = Vec<ListAppsElement>;

pub(crate) async fn list_apps(
    State(state): State<Arc<SharedState>>,
) -> Result<Json<ListAppsResponse>, StatusCode> {
    let apps = sqlx::query_as!(ListAppsElement, "SELECT id, name, description FROM apps")
        .fetch_all(&state.db)
        .await
        .map_err(|err| {
            error!(error = ?err, "failed to get apps from database");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(apps))
}

/// ==================== POST /apps ====================
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateAppRequest {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateAppResponse {
    pub id: i64,
}

pub(crate) async fn create_app(
    State(state): State<Arc<SharedState>>,
    Json(body): Json<CreateAppRequest>,
) -> Result<Json<CreateAppResponse>, StatusCode> {
    let res = sqlx::query!(
        "INSERT INTO apps (name, description) VALUES (?, ?) RETURNING id",
        body.name,
        body.description
    )
    .fetch_one(&state.db)
    .await
    .map_err(|err| match err {
        // Unique violations are client errors,
        sqlx::Error::Database(err) if err.is_unique_violation() => StatusCode::CONFLICT,
        err => {
            error!(error = ?err, "failed to add app to database");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;

    Ok(Json(CreateAppResponse { id: res.id }))
}

/// ==================== GET /apps/{id} ====================
#[derive(Debug, Deserialize, Serialize)]
pub struct GetAppResponse {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
}

pub(crate) async fn get_app(
    State(state): State<Arc<SharedState>>,
    Path(id): Path<i64>,
) -> Result<Json<GetAppResponse>, StatusCode> {
    let app = sqlx::query_as!(
        GetAppResponse,
        "SELECT id, name, description FROM apps WHERE id = ? LIMIT 1",
        id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|err| {
        error!(error = ?err, "failed to get app from database");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match app {
        Some(app) => Ok(Json(app)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// ==================== PUT /apps/{id} ====================
#[derive(Debug, Deserialize, Serialize)]
pub struct PutAppRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PutAppResponse {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
}

pub(crate) async fn update_app(
    State(state): State<Arc<SharedState>>,
    Path(id): Path<i64>,
    Json(body): Json<PutAppRequest>,
) -> Result<Json<PutAppResponse>, StatusCode> {
    let app = sqlx::query_as!(
        PutAppResponse,
        "UPDATE apps SET name = $2, description = $3 WHERE id = $1 RETURNING id, name, description",
        id,
        body.name,
        body.description,
    )
    .fetch_one(&state.db)
    .await
    .map_err(|err| match err {
        sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
        sqlx::Error::Database(err) if err.is_unique_violation() => StatusCode::CONFLICT,
        err => {
            error!(error = ?err, "failed to get app from database");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;

    Ok(Json(app))
}

/// ==================== DELETE /apps/{id} ====================
pub(crate) async fn delete_app(
    State(state): State<Arc<SharedState>>,
    Path(id): Path<i64>,
) -> StatusCode {
    let res = sqlx::query!("DELETE FROM apps WHERE id = ?", id)
        .execute(&state.db)
        .await;

    match res {
        Ok(_) => StatusCode::OK,
        Err(err) => {
            error!(error = ?err, "failed to delete app from database");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
