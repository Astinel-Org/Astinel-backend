use rand::Rng;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// In-memory nonce store for wallet challenge requests.
#[derive(Clone)]
pub struct NonceStore {
    inner: Arc<Mutex<HashMap<String, NonceEntry>>>,
}

#[derive(Clone)]
struct NonceEntry {
    nonce: String,
    expires_at: Instant,
}

impl Default for NonceStore {
    fn default() -> Self {
        Self::new()
    }
}

impl NonceStore {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Generate and store a nonce for a given stellar public key.
    /// Returns the nonce string.
    pub async fn generate(&self, public_key: &str) -> String {
        let nonce: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        let mut store = self.inner.lock().await;
        store.insert(
            public_key.to_string(),
            NonceEntry {
                nonce: nonce.clone(),
                expires_at: Instant::now() + Duration::from_secs(300),
            },
        );
        nonce
    }

    /// Consume and remove a nonce. Returns None if not found or expired.
    pub async fn consume(&self, public_key: &str) -> Option<String> {
        let mut store = self.inner.lock().await;
        let entry = store.remove(public_key)?;
        if entry.expires_at < Instant::now() {
            return None;
        }
        Some(entry.nonce)
    }
}

/// Decode a Stellar G... address to raw ed25519 public key bytes.
pub fn stellar_pk_to_bytes(public_key: &str) -> Result<[u8; 32], String> {
    let bytes = stellar_strkey::Strkey::from_string(public_key)
        .map_err(|e| format!("Invalid Stellar public key: {}", e))?;

    match bytes {
        stellar_strkey::Strkey::PublicKeyEd25519(pk) => Ok(pk.0),
        _ => Err("Not a Stellar public key".to_string()),
    }
}

/// Verify a signature against a Stellar public key and message.
pub fn verify_signature(public_key: &str, message: &[u8], signature: &[u8]) -> Result<(), String> {
    let pk_bytes = stellar_pk_to_bytes(public_key)?;
    let pk = ed25519_dalek::VerifyingKey::from_bytes(&pk_bytes)
        .map_err(|e| format!("Invalid public key bytes: {}", e))?;

    let sig = ed25519_dalek::Signature::from_slice(signature)
        .map_err(|e| format!("Invalid signature: {}", e))?;

    pk.verify_strict(message, &sig)
        .map_err(|_| "Signature verification failed".to_string())
}

/// Build the challenge message a user must sign.
pub fn build_challenge_message(nonce: &str) -> String {
    format!("Astinel login: {}", nonce)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::Signer;
    use stellar_strkey::ed25519::PublicKey as StellarPublicKey;

    fn encode_pk(pk: &ed25519_dalek::VerifyingKey) -> String {
        let pk_bytes = ed25519_dalek::VerifyingKey::to_bytes(pk);
        StellarPublicKey(pk_bytes).to_string().as_str().to_string()
    }

    #[test]
    fn test_pk_roundtrip() {
        let mut rng = rand::thread_rng();
        let sk = ed25519_dalek::SigningKey::generate(&mut rng);
        let pk = sk.verifying_key();

        let encoded = encode_pk(&pk);
        assert!(encoded.starts_with("G"));

        let decoded = stellar_pk_to_bytes(&encoded).unwrap();
        assert_eq!(pk.to_bytes(), decoded);
    }

    #[test]
    fn test_sign_and_verify() {
        let mut rng = rand::thread_rng();
        let sk = ed25519_dalek::SigningKey::generate(&mut rng);
        let pk = sk.verifying_key();

        let encoded = encode_pk(&pk);
        let message = b"test message";
        let signature = sk.sign(message);

        let result = verify_signature(&encoded, message, &signature.to_bytes());
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_wrong_key() {
        let mut rng = rand::thread_rng();
        let sk = ed25519_dalek::SigningKey::generate(&mut rng);
        let pk = sk.verifying_key();
        let encoded = encode_pk(&pk);
        let message = b"test message";

        let other_sk = ed25519_dalek::SigningKey::generate(&mut rng);
        let signature = other_sk.sign(message);

        let result = verify_signature(&encoded, message, &signature.to_bytes());
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_pk() {
        let result = stellar_pk_to_bytes("GABC123");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_nonce_store() {
        let store = NonceStore::new();
        let pk = "GA7QNFM7J2Z3M6Q3KQ6Y7R6XYZ";

        let nonce = store.generate(pk).await;
        assert_eq!(nonce.len(), 32);

        let consumed = store.consume(pk).await;
        assert_eq!(consumed, Some(nonce));

        let again = store.consume(pk).await;
        assert_eq!(again, None);
    }
}
