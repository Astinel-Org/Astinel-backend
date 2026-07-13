use axum::{Router, routing::post, Json, extract::State};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::state::AppState;
use crate::api::response::ApiResponse;
use crate::api::errors::ApiError;
use crate::auth::{AuthTokens, JwtService, PasswordService, AuthError};
use crate::database::models::user::UserRole;
use crate::database::models::{Organization, OrganizationMember};
use crate::database::repositories::{UserRepository, OrganizationRepository};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub tokens: AuthTokens,
    pub user_id: String,
    pub role: String,
}

async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, ApiError> {
    let user = state.user_repository
        .find_by_email(&req.email)
        .await?
        .ok_or(ApiError::Auth(AuthError::InvalidCredentials))?;

    let valid = PasswordService::verify(&req.password, &user.password_hash)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    if !valid {
        return Err(ApiError::Auth(AuthError::InvalidCredentials));
    }

    let tokens = state.jwt_service
        .issue_tokens(user.id, &user.email, &user.role.to_string(), None)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(ApiResponse::ok(LoginResponse {
        tokens,
        user_id: user.id.to_string(),
        role: user.role.to_string(),
    }))
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub display_name: String,
}

async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, ApiError> {
    let existing = state.user_repository
        .find_by_email(&req.email)
        .await?;
    if existing.is_some() {
        return Err(ApiError::Auth(AuthError::EmailAlreadyExists));
    }

    let password_hash = PasswordService::hash(&req.password)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let role = UserRole::User;
    let user = crate::database::models::User::new(
        req.email,
        password_hash,
        req.display_name,
        role.clone(),
    );

    let saved = state.user_repository
        .create(&user)
        .await?;

    // Auto-create a personal organization for the new user
    let base_slug = slugify(&saved.display_name);
    let slug = format!("{}-{}", base_slug, &saved.id.to_string()[..8]);
    let org = Organization::new(
        format!("{}'s Organization", saved.display_name),
        slug,
        saved.id,
        "free".to_string(),
    );
    let created_org = state.organization_repository
        .create(&org)
        .await?;

    // Add user as Owner member
    let member = OrganizationMember::new(
        created_org.id,
        saved.id,
        "owner".to_string(),
    );
    sqlx::query(
        "INSERT INTO organization_members (id, organization_id, user_id, role, invited_by, joined_at, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(member.id)
    .bind(member.organization_id)
    .bind(member.user_id)
    .bind(&member.role)
    .bind(member.invited_by)
    .bind(member.joined_at)
    .bind(member.created_at)
    .bind(member.updated_at)
    .execute(&state.pool)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    let tokens = state.jwt_service
        .issue_tokens(saved.id, &saved.email, &role.to_string(), Some(created_org.id))
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(ApiResponse::ok(LoginResponse {
        tokens,
        user_id: saved.id.to_string(),
        role: role.to_string(),
    }))
}

fn slugify(name: &str) -> String {
    let slug: String = name.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_' || *c == ' ')
        .map(|c| if c == ' ' { '-' } else { c })
        .collect();
    let trimmed = slug.trim_matches('-').to_string();
    if trimmed.is_empty() { "user".to_string() } else { trimmed }
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

async fn refresh(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<ApiResponse<AuthTokens>>, ApiError> {
    let claims = state.jwt_service
        .validate_refresh_token(&req.refresh_token)
        .map_err(|_| ApiError::Auth(AuthError::RefreshExpired))?;

    let tokens = state.jwt_service
        .issue_tokens(claims.sub, &claims.email, &claims.role, claims.org_id)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(ApiResponse::ok(tokens))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/auth/login", post(login))
        .route("/v1/auth/register", post(register))
        .route("/v1/auth/refresh", post(refresh))
}
