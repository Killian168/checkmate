use base64::{engine::general_purpose, Engine as _};
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use reqwest::Client;
use std::collections::HashMap;
use tracing::info;

use crate::models::Jwks;

pub struct AuthService {
    pub client: Client,
    pub jwks_url: String,
    pub client_id: String,
    pub issuer: String,
}

impl AuthService {
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let jwks_url = std::env::var("JWKS_URL").map_err(|_| "JWKS_URL not set")?;
        let client_id = std::env::var("COGNITO_USER_POOL_CLIENT_ID")
            .map_err(|_| "COGNITO_USER_POOL_CLIENT_ID not set")?;
        let issuer = std::env::var("ISSUER").map_err(|_| "ISSUER not set")?;

        info!("AuthService initialized with JWKS_URL: {}", jwks_url);

        Ok(AuthService {
            client: Client::new(),
            jwks_url,
            client_id,
            issuer,
        })
    }

    pub async fn verify_id_token(
        &self,
        token: &str,
    ) -> Result<shared::auth::Claims, Box<dyn std::error::Error + Send + Sync>> {
        info!("Verifying ID token");

        let jwt_parts: Vec<&str> = token.split('.').collect();
        if jwt_parts.len() != 3 {
            return Err("Invalid JWT format".into());
        }

        // Decode header to get kid
        let header_b64 = jwt_parts[0];
        let header_json = general_purpose::URL_SAFE_NO_PAD.decode(header_b64)?;
        let header: HashMap<String, serde_json::Value> = serde_json::from_slice(&header_json)?;
        let kid = header
            .get("kid")
            .and_then(|v| v.as_str())
            .ok_or("No kid in header")?;

        info!("Token kid: {}", kid);

        // Fetch JWKs
        let jwks: Jwks = self.client.get(&self.jwks_url).send().await?.json().await?;
        info!("Fetched JWKs with {} keys", jwks.keys.len());

        // Find matching key
        let jwk = jwks
            .keys
            .iter()
            .find(|k| k.kid == kid)
            .ok_or("No matching key found")?;
        if jwk.kty != "RSA" {
            return Err("Unsupported key type".into());
        }

        // Create decoding key from JWK components
        let decoding_key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)?;

        // Validate JWT for ID token
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[self.issuer.clone()]);
        validation.set_audience(&[self.client_id.clone()]);

        let token_data =
            jsonwebtoken::decode::<shared::auth::Claims>(token, &decoding_key, &validation)?;

        // Check if token is expired
        let now = chrono::Utc::now().timestamp() as u64;
        if token_data.claims.exp < now {
            return Err("Token expired".into());
        }

        info!(
            "Token verified successfully for user: {}",
            token_data.claims.sub
        );
        Ok(token_data.claims)
    }
}
