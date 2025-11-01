use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct AuthError {
    pub message: String,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        (StatusCode::UNAUTHORIZED, Json(self)).into_response()
    }
}

/// Extractor for authenticated user information
///
/// API Gateway validates the JWT and passes the claims in the request context.
/// This extractor retrieves those claims from the Lambda event.
pub struct AuthenticatedUser {
    pub claims: shared::auth::Claims,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract JWT from Authorization header
        let auth_header = parts.headers.get(AUTHORIZATION).ok_or_else(|| AuthError {
            message: "Missing Authorization header".to_string(),
        })?;
        let auth_str = auth_header.to_str().map_err(|_| AuthError {
            message: "Invalid Authorization header".to_string(),
        })?;
        if !auth_str.starts_with("Bearer ") {
            return Err(AuthError {
                message: "Invalid Authorization format".to_string(),
            });
        }
        let token = &auth_str[7..]; // remove "Bearer "
        let claims = shared::auth::extract_claims(token).map_err(|e| AuthError {
            message: format!("Invalid JWT: {}", e),
        })?;
        Ok(AuthenticatedUser { claims })
    }
}
