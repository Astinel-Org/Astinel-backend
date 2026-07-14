use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use crate::auth::AuthContext;

impl<S: Send + Sync> FromRequestParts<S> for AuthContext {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let context = parts
            .extensions
            .get::<AuthContext>()
            .cloned()
            .unwrap_or_else(AuthContext::anonymous);
        Ok(context)
    }
}
