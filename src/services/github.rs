use jsonwebtoken::{encode, EncodingKey, Header};
use octocrab::Octocrab;
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub struct GitHubConfig {
    pub app_id: String,
    pub private_key: String,
    pub webhook_secret: String,
}

impl GitHubConfig {
    pub fn from_env() -> Option<Self> {
        let app_id = std::env::var("GITHUB_APP_ID").ok()?;
        let private_key = std::env::var("GITHUB_APP_PRIVATE_KEY").ok()?;
        let webhook_secret = std::env::var("GITHUB_WEBHOOK_SECRET").ok()?;
        Some(Self {
            app_id,
            private_key,
            webhook_secret,
        })
    }
}

#[derive(Serialize)]
struct AppJwtClaims {
    iss: String,
    iat: usize,
    exp: usize,
}

pub struct GitHubService {
    config: GitHubConfig,
}

impl GitHubService {
    pub fn new(config: GitHubConfig) -> Self {
        Self { config }
    }

    pub fn generate_app_jwt(&self) -> Result<String, jsonwebtoken::errors::Error> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;

        let claims = AppJwtClaims {
            iss: self.config.app_id.clone(),
            iat: now - 60,
            exp: now + 600,
        };

        encode(
            &Header::new(jsonwebtoken::Algorithm::RS256),
            &claims,
            &EncodingKey::from_rsa_pem(self.config.private_key.as_bytes())?,
        )
    }

    pub async fn get_installation_token(&self, installation_id: i64) -> Result<String, String> {
        let jwt = self.generate_app_jwt().map_err(|e| e.to_string())?;

        let octocrab = Octocrab::builder()
            .add_header("Authorization".parse().unwrap(), format!("Bearer {}", jwt))
            .build()
            .map_err(|e| e.to_string())?;

        let resp: serde_json::Value = octocrab
            .post(
                format!("/app/installations/{}/access_tokens", installation_id),
                None::<&()>,
            )
            .await
            .map_err(|e| e.to_string())?;

        resp["token"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "No token in response".to_string())
    }

    pub fn client_for_token(token: &str) -> Result<Octocrab, String> {
        Octocrab::builder()
            .personal_token(token.to_string())
            .build()
            .map_err(|e| e.to_string())
    }

    pub async fn list_repositories(
        &self,
        installation_id: i64,
    ) -> Result<Vec<serde_json::Value>, String> {
        let token = self.get_installation_token(installation_id).await?;
        let octocrab = Self::client_for_token(&token)?;

        let body: serde_json::Value = octocrab
            .get("/installation/repositories", None::<&()>)
            .await
            .map_err(|e| e.to_string())?;

        Ok(body["repositories"].as_array().cloned().unwrap_or_default())
    }

    pub async fn get_repository(
        &self,
        installation_id: i64,
        owner: &str,
        repo: &str,
    ) -> Result<serde_json::Value, String> {
        let token = self.get_installation_token(installation_id).await?;
        let octocrab = Self::client_for_token(&token)?;

        octocrab
            .get::<serde_json::Value, _, _>(&format!("/repos/{}/{}", owner, repo), None::<&()>)
            .await
            .map_err(|e| e.to_string())
    }

    pub fn verify_webhook_signature(payload: &[u8], signature_header: &str, secret: &[u8]) -> bool {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        let expected = signature_header
            .strip_prefix("sha256=")
            .unwrap_or(signature_header);

        let mut mac = Hmac::<Sha256>::new_from_slice(secret).expect("HMAC key");
        mac.update(payload);
        let computed: String = mac
            .finalize()
            .into_bytes()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect();

        computed == expected
    }
}
