use async_trait::async_trait;

#[async_trait]
pub trait AiProvider: Send + Sync {
    async fn generate_fix_suggestion(&self, finding_context: &str, code_snippet: &str) -> Result<String, String>;
    async fn analyze_security(&self, query: &str, scan_summary: &str) -> Result<String, String>;
    async fn health(&self) -> Result<bool, String>;
}

pub struct OllamaProvider {
    base_url: String,
    model: String,
    client: reqwest::Client,
}

impl OllamaProvider {
    pub fn new(base_url: Option<String>, model: Option<String>) -> Self {
        Self {
            base_url: base_url.unwrap_or_else(|| "http://localhost:11434".to_string()),
            model: model.unwrap_or_else(|| "llama3.2".to_string()),
            client: reqwest::Client::new(),
        }
    }

    pub fn from_env() -> Self {
        Self::new(
            std::env::var("OLLAMA_URL").ok(),
            std::env::var("OLLAMA_MODEL").ok(),
        )
    }

    async fn generate(&self, system_prompt: &str, user_prompt: &str) -> Result<String, String> {
        let url = format!("{}/api/generate", self.base_url);
        let body = serde_json::json!({
            "model": self.model,
            "system": system_prompt,
            "prompt": user_prompt,
            "stream": false,
        });

        let resp = self.client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Ollama request failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("Ollama returned status: {}", resp.status()));
        }

        let data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;

        data["response"]
            .as_str()
            .map(|s| s.trim().to_string())
            .ok_or_else(|| "Empty response from Ollama".to_string())
    }
}

#[async_trait]
impl AiProvider for OllamaProvider {
    async fn generate_fix_suggestion(&self, finding_context: &str, code_snippet: &str) -> Result<String, String> {
        let system = "You are a security expert for Stellar Soroban smart contracts. \
                      Given a finding and the relevant code, suggest a specific fix. \
                      Be concise and provide code examples.";

        let prompt = format!(
            "Finding: {}\n\nRelevant code:\n```rust\n{}\n```\n\nProvide a specific fix suggestion with code example.",
            finding_context, code_snippet,
        );

        self.generate(system, &prompt).await
    }

    async fn analyze_security(&self, query: &str, scan_summary: &str) -> Result<String, String> {
        let system = "You are a security analyst for Stellar Soroban smart contracts. \
                      Answer questions about scan results clearly and concisely. \
                      Focus on actionable security insights.";

        let prompt = format!(
            "Scan results:\n{}\n\nQuestion: {}\n\nProvide a clear security analysis.",
            scan_summary, query,
        );

        self.generate(system, &prompt).await
    }

    async fn health(&self) -> Result<bool, String> {
        let url = format!("{}/api/tags", self.base_url);
        let resp = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Ollama health check failed: {}", e))?;

        Ok(resp.status().is_success())
    }
}

pub struct DisabledProvider;

#[async_trait]
impl AiProvider for DisabledProvider {
    async fn generate_fix_suggestion(&self, _finding_context: &str, _code_snippet: &str) -> Result<String, String> {
        Err("AI provider is not configured".to_string())
    }

    async fn analyze_security(&self, _query: &str, _scan_summary: &str) -> Result<String, String> {
        Err("AI provider is not configured".to_string())
    }

    async fn health(&self) -> Result<bool, String> {
        Ok(false)
    }
}

pub fn create_provider() -> Box<dyn AiProvider> {
    if std::env::var("OLLAMA_URL").is_ok() || std::env::var("AI_PROVIDER").map(|v| v == "ollama").unwrap_or(false) {
        Box::new(OllamaProvider::from_env())
    } else {
        Box::new(DisabledProvider)
    }
}
