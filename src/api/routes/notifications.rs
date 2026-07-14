use axum::{Router, routing::{get, post}, Json, extract::{State, Path, Query}};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::state::AppState;
use crate::auth::AuthContext;
use crate::api::response::ApiResponse;
use crate::api::errors::ApiError;
use crate::database::repositories::notification_repository::NotificationRepository;

#[derive(Serialize)]
pub struct NotificationResponse {
    pub id: String,
    pub organization_id: String,
    pub event_type: String,
    pub title: String,
    pub message: String,
    pub severity: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub is_read: bool,
    pub created_at: String,
}

#[derive(Deserialize)]
pub struct NotificationsQuery {
    pub limit: Option<i64>,
}

#[derive(Serialize)]
pub struct UnreadCountResponse {
    pub count: i64,
}

async fn list_notifications(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Query(query): Query<NotificationsQuery>,
) -> Result<Json<Vec<NotificationResponse>>, ApiError> {
    if !auth.is_authenticated {
        return Err(ApiError::Auth(crate::auth::AuthError::PermissionDenied));
    }
    let org_id = auth.org_id.unwrap_or_default();
    let limit = query.limit.unwrap_or(50).min(200);
    let events = state.notification_repository.list_by_organization(org_id, limit).await?;

    let resp = events.into_iter().map(|e| NotificationResponse {
        id: e.id.to_string(),
        organization_id: e.organization_id.to_string(),
        event_type: e.event_type,
        title: e.title,
        message: e.message,
        severity: e.severity,
        resource_type: e.resource_type,
        resource_id: e.resource_id.map(|id| id.to_string()),
        is_read: e.is_read,
        created_at: e.created_at.to_rfc3339(),
    }).collect();
    Ok(Json(resp))
}

async fn count_unread(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
) -> Result<Json<ApiResponse<UnreadCountResponse>>, ApiError> {
    if !auth.is_authenticated {
        return Err(ApiError::Auth(crate::auth::AuthError::PermissionDenied));
    }
    let org_id = auth.org_id.unwrap_or_default();
    let count = state.notification_repository.count_unread(org_id).await?;
    Ok(ApiResponse::ok(UnreadCountResponse { count }))
}

async fn mark_read(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    if !auth.is_authenticated {
        return Err(ApiError::Auth(crate::auth::AuthError::PermissionDenied));
    }
    state.notification_repository.mark_read(id).await?;
    Ok(ApiResponse::ok(()))
}

async fn mark_all_read(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    if !auth.is_authenticated {
        return Err(ApiError::Auth(crate::auth::AuthError::PermissionDenied));
    }
    let org_id = auth.org_id.unwrap_or_default();
    state.notification_repository.mark_all_read(org_id).await?;
    Ok(ApiResponse::ok(()))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/notifications", get(list_notifications))
        .route("/v1/notifications/unread/count", get(count_unread))
        .route("/v1/notifications/{id}/read", post(mark_read))
        .route("/v1/notifications/read-all", post(mark_all_read))
}
