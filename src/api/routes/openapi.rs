use crate::state::AppState;
use axum::{routing::get, Json, Router};
use serde_json::Value;
use std::sync::Arc;

async fn get_openapi() -> Json<Value> {
    Json(serde_json::json!({
        "openapi": "3.1.0",
        "info": {
            "title": "Astinel API",
            "description": "Security scanning platform for Stellar and Soroban smart contracts",
            "version": "0.5.0",
            "contact": {
                "name": "Astinel",
                "url": "https://astinel.dev"
            }
        },
        "servers": [
            {
                "url": "/api/v1",
                "description": "API v1"
            }
        ],
        "paths": {
            "/auth/register": {
                "post": { "summary": "Register a new user", "tags": ["Auth"] }
            },
            "/auth/login": {
                "post": { "summary": "Login with email/password", "tags": ["Auth"] }
            },
            "/auth/wallet/challenge": {
                "post": { "summary": "Get a wallet challenge nonce", "tags": ["Auth"] }
            },
            "/auth/wallet/verify": {
                "post": { "summary": "Verify wallet signature", "tags": ["Auth"] }
            },
            "/health": {
                "get": { "summary": "Health check", "tags": ["System"] }
            },
            "/version": {
                "get": { "summary": "Get version info", "tags": ["System"] }
            },
            "/dashboard": {
                "get": { "summary": "Organization dashboard stats", "tags": ["Dashboard"] }
            },
            "/projects": {
                "get": { "summary": "List projects", "tags": ["Projects"] },
                "post": { "summary": "Create a project", "tags": ["Projects"] }
            },
            "/projects/{id}": {
                "get": { "summary": "Get project details", "tags": ["Projects"] },
                "put": { "summary": "Update project", "tags": ["Projects"] },
                "delete": { "summary": "Delete project", "tags": ["Projects"] }
            },
            "/scans": {
                "get": { "summary": "List scans", "tags": ["Scans"] },
                "post": { "summary": "Trigger a scan", "tags": ["Scans"] }
            },
            "/scans/{id}": {
                "get": { "summary": "Get scan details", "tags": ["Scans"] }
            },
            "/scans/{id}/result": {
                "get": { "summary": "Get scan result", "tags": ["Scans"] }
            },
            "/scans/{id}/progress": {
                "get": { "summary": "Get scan progress", "tags": ["Scans"] }
            },
            "/scans/{id}/cancel": {
                "post": { "summary": "Cancel a scan", "tags": ["Scans"] }
            },
            "/scans/{id}/retry": {
                "post": { "summary": "Retry a scan", "tags": ["Scans"] }
            },
            "/findings": {
                "get": { "summary": "List findings", "tags": ["Findings"] }
            },
            "/findings/{id}": {
                "get": { "summary": "Get finding details", "tags": ["Findings"] },
                "patch": { "summary": "Update finding (suppress/resolve)", "tags": ["Findings"] }
            },
            "/reports": {
                "get": { "summary": "List reports", "tags": ["Reports"] }
            },
            "/reports/{id}": {
                "get": { "summary": "Get report details", "tags": ["Reports"] }
            },
            "/reports/{format}/{scanResultId}": {
                "get": { "summary": "Download report by format", "tags": ["Reports"] }
            },
            "/notifications": {
                "get": { "summary": "List notifications", "tags": ["Notifications"] }
            },
            "/notifications/unread/count": {
                "get": { "summary": "Count unread notifications", "tags": ["Notifications"] }
            },
            "/notifications/{id}/read": {
                "post": { "summary": "Mark notification as read", "tags": ["Notifications"] }
            },
            "/notifications/read-all": {
                "post": { "summary": "Mark all as read", "tags": ["Notifications"] }
            },
            "/webhooks/github": {
                "post": { "summary": "GitHub App webhook receiver", "tags": ["Webhooks"] }
            }
        },
        "tags": [
            { "name": "Auth", "description": "Authentication endpoints" },
            { "name": "Dashboard", "description": "Dashboard statistics" },
            { "name": "Projects", "description": "Project management" },
            { "name": "Scans", "description": "Security scan operations" },
            { "name": "Findings", "description": "Scan findings management" },
            { "name": "Reports", "description": "Report generation and download" },
            { "name": "Notifications", "description": "Notification events" },
            { "name": "Webhooks", "description": "External webhooks" },
            { "name": "System", "description": "System endpoints" }
        ]
    }))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/openapi.json", get(get_openapi))
}
