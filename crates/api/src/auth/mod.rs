use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};

/// Claims extracted from the Cognito JWT token by API Gateway
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: Option<String>,
    #[serde(rename = "cognito:username")]
    pub cognito_username: Option<String>,
    pub exp: u64,
    pub iat: u64,
    pub token_use: Option<String>,
    #[serde(rename = "email_verified")]
    pub email_verified: Option<bool>,
    pub iss: Option<String>,
    pub aud: Option<String>,
    #[serde(rename = "event_id")]
    pub event_id: Option<String>,
    #[serde(rename = "auth_time")]
    pub auth_time: Option<u64>,
    pub jti: Option<String>,
}

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
    pub claims: Claims,
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
        let jwt_parts: Vec<&str> = token.split('.').collect();
        if jwt_parts.len() != 3 {
            return Err(AuthError {
                message: "Invalid JWT format".to_string(),
            });
        }
        let payload = general_purpose::URL_SAFE_NO_PAD
            .decode(jwt_parts[1])
            .map_err(|_| AuthError {
                message: "Invalid JWT payload".to_string(),
            })?;
        let claims: Claims = serde_json::from_slice(&payload).map_err(|_| AuthError {
            message: "Invalid claims JSON".to_string(),
        })?;
        Ok(AuthenticatedUser { claims })
    }
}
