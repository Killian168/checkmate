use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use lambda_http::tracing::warn;

use crate::{models::AppState, services::errors::auth_service_errors::AuthServiceError};

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: String,
}

impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Extract Authorization header
        let auth_header = parts
            .headers
            .get("Authorization")
            .ok_or(StatusCode::UNAUTHORIZED)?
            .to_str()
            .map_err(|_| StatusCode::BAD_REQUEST)?;

        // Check if it starts with "Bearer "
        if !auth_header.starts_with("Bearer ") {
            warn!("Invalid Authorization header format");
            return Err(StatusCode::UNAUTHORIZED);
        }

        // Extract the token (remove "Bearer " prefix)
        let token = &auth_header[7..];

        // Verify JWT and extract user ID
        let user_id = match state.auth_service.extract_user_id_from_token(token) {
            Ok(id) => id,
            Err(e) => match e {
                AuthServiceError::InvalidToken => {
                    warn!("Invalid JWT token provided");
                    return Err(StatusCode::UNAUTHORIZED);
                }
                AuthServiceError::ExpiredToken => {
                    warn!("Expired JWT token provided");
                    return Err(StatusCode::UNAUTHORIZED);
                }
                AuthServiceError::JwtError(_) => {
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
                _ => {
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            },
        };

        Ok(AuthenticatedUser { user_id })
    }
}
