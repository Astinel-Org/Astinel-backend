use axum::{Router, routing::post, Json, extract::State};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use crate::state::AppState;
use crate::api::response::ApiResponse;
use crate::api::errors::ApiError;
use crate::auth::{AuthTokens, AuthError};
use crate::database::models::user::UserRole;
use crate::database::repositories::{UserRepository, OrganizationRepository};

#[derive(Deserialize)]
pub struct ChallengeRequest {
    pub public_key: String,
}

#[derive(Serialize)]
pub struct ChallengeResponse {
    pub nonce: String,
    pub message: String,
}

async fn challenge(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ChallengeRequest>,
) -> Result<Json<ApiResponse<ChallengeResponse>>, ApiError> {
    let nonce = state.nonce_store.generate(&req.public_key).await;
    let message = crate::auth::wallet::build_challenge_message(&nonce);

    Ok(ApiResponse::ok(ChallengeResponse { nonce, message }))
}

#[derive(Deserialize)]
pub struct WalletLoginRequest {
    pub public_key: String,
    pub signed_message: String,
}

#[derive(Serialize)]
pub struct WalletLoginResponse {
    pub tokens: AuthTokens,
    pub user_id: String,
    pub is_new_user: bool,
}

async fn wallet_login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<WalletLoginRequest>,
) -> Result<Json<ApiResponse<WalletLoginResponse>>, ApiError> {
    let nonce = state.nonce_store.consume(&req.public_key).await
        .ok_or_else(|| ApiError::Auth(AuthError::InvalidCredentials))?;

    let message = crate::auth::wallet::build_challenge_message(&nonce);

    let signature = BASE64.decode(req.signed_message.as_bytes())
        .map_err(|_| ApiError::BadRequest("Invalid base64 signature".to_string()))?;

    crate::auth::wallet::verify_signature(&req.public_key, message.as_bytes(), &signature)
        .map_err(|_| ApiError::Auth(AuthError::InvalidCredentials))?;

    let existing = state.user_repository
        .find_by_stellar_public_key(&req.public_key)
        .await?;

    let (user_id, is_new_user) = if let Some(user) = existing {
        (user.id, false)
    } else {
        let email = format!("wallet-{}@astinel.io", &req.public_key[..12]);
        let password_hash = crate::auth::PasswordService::hash(&uuid::Uuid::new_v4().to_string())
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        let display_name = format!("Wallet {}", &req.public_key[..8]);

        let mut user = crate::database::models::User::new(
            email,
            password_hash,
            display_name,
            UserRole::User,
        );
        user.stellar_public_key = Some(req.public_key.clone());

        let saved = state.user_repository
            .create(&user)
            .await?;

        let slug = format!("wallet-{}", &saved.id.to_string()[..8]);
        let org = crate::database::models::Organization::new(
            format!("{}'s Organization", saved.display_name),
            slug,
            saved.id,
            "free".to_string(),
        );
        let created_org = state.organization_repository
            .create(&org)
            .await?;

        let member = crate::database::models::OrganizationMember::new(
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

        (saved.id, true)
    };

    let tokens = state.jwt_service
        .issue_tokens(user_id, "wallet@astinel.io", "user", None)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(ApiResponse::ok(WalletLoginResponse {
        tokens,
        user_id: user_id.to_string(),
        is_new_user,
    }))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/auth/wallet/challenge", post(challenge))
        .route("/v1/auth/wallet/login", post(wallet_login))
}
