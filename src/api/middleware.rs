use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use std::time::Instant;
use crate::auth::{AuthContext, rbac::Role};
use crate::database::repositories::ApiKeyRepository;
use crate::state::AppState;

fn record_request_metrics(path: &str, method: &str, status: u16, duration_ms: f64) {
    let path_owned = path.to_string();
    let method_owned = method.to_string();
    let path_label: String = if path_owned.starts_with("/metrics") || path_owned.starts_with("/openapi.json") {
        String::from("internal")
    } else {
        path_owned
    };
    metrics::counter!("api_requests_total", "path" => path_label.clone(), "method" => method_owned.clone()).increment(1);
    metrics::histogram!("api_request_duration_ms", "path" => path_label, "method" => method_owned).record(duration_ms);
    if status >= 500 {
        metrics::counter!("api_errors_total", "code" => status.to_string()).increment(1);
    }
}

fn sha256_hex(input: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

async fn authenticate_api_key(state: &AppState, key: &str) -> Option<AuthContext> {
    let hash = sha256_hex(key);
    let api_key = state.api_key_repository.find_by_hash(&hash).await.ok()??;

    let role = "developer".parse::<Role>().unwrap_or(Role::Developer);
    let context = AuthContext::new(
        uuid::Uuid::nil(),
        format!("api:{}", api_key.name),
        role,
        Some(api_key.organization_id),
    );

    let _ = state.api_key_repository.update_last_used(api_key.id).await;
    Some(context)
}

pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path().to_string();
    let method = request.method().to_string();
    let start = Instant::now();

    let api_key_header = request
        .headers()
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok());

    if let Some(key) = api_key_header {
        if let Some(context) = authenticate_api_key(&state, key).await {
            request.extensions_mut().insert(context);
            let response = next.run(request).await;
            let duration = start.elapsed().as_secs_f64() * 1000.0;
            record_request_metrics(&path, &method, response.status().as_u16(), duration);
            return response;
        }
    }

    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    let context = match auth_header {
        Some(token) => {
            match state.jwt_service.validate_access_token(token) {
                Ok(claims) => {
                    let role = claims.role.parse::<Role>()
                        .unwrap_or(Role::Viewer);
                    AuthContext::new(claims.sub, claims.email, role, claims.org_id)
                }
                Err(_) => AuthContext::anonymous(),
            }
        }
        None => AuthContext::anonymous(),
    };

    request.extensions_mut().insert(context);
    let response = next.run(request).await;
    let duration = start.elapsed().as_secs_f64() * 1000.0;
    record_request_metrics(&path, &method, response.status().as_u16(), duration);
    response
}
