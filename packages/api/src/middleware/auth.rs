use axum::{extract::FromRequestParts, http::request::Parts};

use crate::{error::ApiError, state::AppState};
use shared::services::auth_service::AuthServiceTrait;
use shared::services::errors::auth_service_errors::AuthServiceError;

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: String,
}

impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Extract Authorization header
        let auth_header = parts
            .headers
            .get("Authorization")
            .ok_or_else(|| ApiError::AuthService(AuthServiceError::InvalidCredentials))?
            .to_str()
            .map_err(|_| {
                ApiError::AuthService(AuthServiceError::ValidationError(
                    "Invalid header format".to_string(),
                ))
            })?;

        // Check if it starts with "Bearer "
        if !auth_header.starts_with("Bearer ") {
            return Err(ApiError::AuthService(AuthServiceError::InvalidCredentials));
        }

        // Extract the token (remove "Bearer " prefix)
        let token = &auth_header[7..];

        // Verify JWT and extract user ID
        let user_id = state
            .auth_service
            .extract_user_id_from_token(token)
            .map_err(|e| ApiError::from(e))?;

        Ok(AuthenticatedUser { user_id })
    }
}
