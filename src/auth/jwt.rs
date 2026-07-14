use std::time::{SystemTime, UNIX_EPOCH};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,        // user id
    pub email: String,
    pub role: String,
    pub org_id: Option<Uuid>,
    pub exp: usize,
    pub iat: usize,
    pub jti: String,      // token id
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

pub struct JwtService {
    access_secret: String,
    refresh_secret: String,
    access_ttl_secs: i64,
    refresh_ttl_secs: i64,
}

impl JwtService {
    pub fn new(access_secret: String, refresh_secret: String) -> Self {
        Self {
            access_secret,
            refresh_secret,
            access_ttl_secs: 3600,        // 1 hour
            refresh_ttl_secs: 2592000,     // 30 days
        }
    }

    pub fn from_env() -> Self {
        Self::new(
            std::env::var("JWT_ACCESS_SECRET").unwrap_or_else(|_| "astinel-access-secret-dev".to_string()),
            std::env::var("JWT_REFRESH_SECRET").unwrap_or_else(|_| "astinel-refresh-secret-dev".to_string()),
        )
    }

    pub fn issue_tokens(&self, user_id: Uuid, email: &str, role: &str, org_id: Option<Uuid>) -> Result<AuthTokens, jsonwebtoken::errors::Error> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize;

        let access_claims = Claims {
            sub: user_id,
            email: email.to_string(),
            role: role.to_string(),
            org_id,
            exp: now + self.access_ttl_secs as usize,
            iat: now,
            jti: Uuid::new_v4().to_string(),
        };

        let refresh_claims = Claims {
            exp: now + self.refresh_ttl_secs as usize,
            ..access_claims.clone()
        };

        let access_token = encode(
            &Header::default(),
            &access_claims,
            &EncodingKey::from_secret(self.access_secret.as_bytes()),
        )?;

        let refresh_token = encode(
            &Header::default(),
            &refresh_claims,
            &EncodingKey::from_secret(self.refresh_secret.as_bytes()),
        )?;

        Ok(AuthTokens {
            access_token,
            refresh_token,
            expires_in: self.access_ttl_secs,
        })
    }

    pub fn validate_access_token(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.access_secret.as_bytes()),
            &Validation::default(),
        )?;
        Ok(data.claims)
    }

    pub fn validate_refresh_token(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.refresh_secret.as_bytes()),
            &Validation::default(),
        )?;
        Ok(data.claims)
    }
}
