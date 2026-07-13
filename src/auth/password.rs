use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::SaltString;
use rand::rngs::OsRng;

pub struct PasswordService;

impl PasswordService {
    pub fn hash(password: &str) -> Result<String, argon2::password_hash::Error> {
        let salt = SaltString::generate(&mut OsRng);
        let hash = Argon2::default().hash_password(password.as_bytes(), &salt)?;
        Ok(hash.to_string())
    }

    pub fn verify(password: &str, hash: &str) -> Result<bool, argon2::password_hash::Error> {
        let parsed = PasswordHash::new(hash)?;
        Ok(Argon2::default().verify_password(password.as_bytes(), &parsed).is_ok())
    }
}
