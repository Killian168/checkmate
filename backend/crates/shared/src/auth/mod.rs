use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};

/// Claims extracted from the Cognito JWT token
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
    pub jti: Option<String>,
    pub auth_time: Option<u64>,
}

/// Extract claims from a JWT token string without verifying signature (assuming already validated by API Gateway)
pub fn extract_claims(token: &str) -> Result<Claims, Box<dyn std::error::Error + Send + Sync>> {
    let jwt_parts: Vec<&str> = token.split('.').collect();
    if jwt_parts.len() != 3 {
        return Err("Invalid JWT format".into());
    }
    let payload = general_purpose::URL_SAFE_NO_PAD
        .decode(jwt_parts[1])
        .map_err(|_| "Invalid JWT payload")?;
    let claims: Claims = serde_json::from_slice(&payload).map_err(|_| "Invalid claims JSON")?;
    Ok(claims)
}
