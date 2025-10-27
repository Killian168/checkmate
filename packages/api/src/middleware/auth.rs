use axum::{extract::FromRequestParts, http::request::Parts};
use lambda_http::request::RequestContext;

use crate::{error::ApiError, state::AppState};

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
}

impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Extract RequestContext from extensions
        let request_context = parts
            .extensions
            .get::<RequestContext>()
            .ok_or_else(|| ApiError::Unauthorized)?;

        // Extract authorizer claims
        let authorizer_result = request_context.authorizer();
        let authorizer = authorizer_result
            .as_ref()
            .ok_or_else(|| ApiError::Unauthorized)?;

        let jwt = authorizer
            .jwt
            .as_ref()
            .ok_or_else(|| ApiError::Unauthorized)?;

        let claims = &jwt.claims;

        let sub = claims.get("sub").ok_or_else(|| ApiError::Unauthorized)?;

        let email = claims
            .get("email")
            .cloned()
            .unwrap_or_else(|| "".to_string());

        let first_name = claims
            .get("given_name")
            .cloned()
            .unwrap_or_else(|| "".to_string());

        let last_name = claims
            .get("family_name")
            .cloned()
            .unwrap_or_else(|| "".to_string());

        Ok(AuthenticatedUser {
            user_id: sub.to_string(),
            email,
            first_name,
            last_name,
        })
    }
}
