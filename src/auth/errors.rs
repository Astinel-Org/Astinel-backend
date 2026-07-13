use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Token expired")]
    TokenExpired,
    #[error("Token invalid")]
    TokenInvalid,
    #[error("Refresh token expired")]
    RefreshExpired,
    #[error("Permission denied")]
    PermissionDenied,
    #[error("User not found")]
    UserNotFound,
    #[error("Email already exists")]
    EmailAlreadyExists,
    #[error("Password hashing error: {0}")]
    PasswordError(String),
    #[error("Token creation error: {0}")]
    TokenError(String),
}
