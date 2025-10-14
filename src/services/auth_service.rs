use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use lambda_http::tracing::{debug, error, warn};
use std::sync::Arc;

use crate::models::auth::responses::{LoginResponse, TokenClaims};
use crate::services::errors::auth_service_errors::AuthServiceError;
use crate::services::errors::user_service_errors::UserServiceError;
use crate::services::user_service::UserService;

pub struct AuthService {
    user_service: Arc<UserService>,
    jwt_secret: String,
}

impl AuthService {
    pub fn new(user_service: Arc<UserService>) -> Self {
        let jwt_secret =
            std::env::var("JWT_SECRET").expect("JWT_SECRET environment variable must be set");
        AuthService {
            user_service,
            jwt_secret,
        }
    }

    pub async fn authenticate_user(
        &self,
        email: &str,
        password: &str,
    ) -> Result<LoginResponse, AuthServiceError> {
        // Validate input parameters
        if email.is_empty() || password.is_empty() {
            return Err(AuthServiceError::ValidationError(
                "Email or password cannot be empty".to_string(),
            ));
        }

        // Get user from UserService by email
        debug!("Attempting to authenticate user: {}", email);
        match self.user_service.get_user_by_email(email).await {
            Ok(user) => {
                debug!("User found, checking password for: {}", email);
                // Check if password matches (in real app, use proper password hashing)
                if user.password == password {
                    debug!("Password match successful for user: {}", email);
                    // Generate JWT token with user ID
                    self.generate_token(&user.id)
                } else {
                    debug!("Password mismatch for user: {}", email);
                    Err(AuthServiceError::InvalidCredentials)
                }
            }
            Err(UserServiceError::UserNotFound) => {
                warn!("User not found during authentication: {}", email);
                Err(AuthServiceError::InvalidCredentials)
            }
            Err(err) => {
                error!(
                    "UserService error during authentication for '{}': {:#?}",
                    email, err
                );
                Err(AuthServiceError::UserServiceError(err))
            }
        }
    }

    fn generate_token(&self, user_id: &str) -> Result<LoginResponse, AuthServiceError> {
        let now = Utc::now();
        let expires_in = 24 * 60 * 60; // 24 hours in seconds
        let exp = (now + Duration::hours(24)).timestamp() as usize;
        let iat = now.timestamp() as usize;

        let claims = TokenClaims {
            sub: user_id.to_string(),
            exp,
            iat,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )
        .map_err(|e| {
            error!("JWT encoding error for user '{}': {:#?}", user_id, e);
            AuthServiceError::JwtError(format!("{:#?}", e))
        })?;

        Ok(LoginResponse {
            token,
            token_type: "Bearer".to_string(),
            expires_in,
        })
    }

    pub fn verify_token(&self, token: &str) -> Result<TokenClaims, AuthServiceError> {
        let decoding_key = DecodingKey::from_secret(self.jwt_secret.as_ref());
        let validation = Validation::default();

        match decode::<TokenClaims>(token, &decoding_key, &validation) {
            Ok(token_data) => {
                let now = Utc::now().timestamp() as usize;
                if token_data.claims.exp < now {
                    debug!("Token expired for user: {}", token_data.claims.sub);
                    Err(AuthServiceError::ExpiredToken)
                } else {
                    Ok(token_data.claims)
                }
            }
            Err(err) => {
                error!("JWT decoding error: {:#?}", err);
                match err.kind() {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                        debug!("Token signature expired");
                        Err(AuthServiceError::ExpiredToken)
                    }
                    _ => {
                        debug!("Invalid token provided");
                        Err(AuthServiceError::InvalidToken)
                    }
                }
            }
        }
    }

    pub fn extract_user_id_from_token(&self, token: &str) -> Result<String, AuthServiceError> {
        let claims = self.verify_token(token)?;
        Ok(claims.sub)
    }

    // Backward compatibility method - kept for migration purposes
    pub fn extract_user_email_from_token(&self, token: &str) -> Result<String, AuthServiceError> {
        // This method is deprecated and should be removed after migration
        // For now, it attempts to extract what might be an email from older tokens
        let claims = self.verify_token(token)?;

        // If the token contains an email (old format), return it
        if claims.sub.contains('@') {
            Ok(claims.sub)
        } else {
            // If it's a UUID (new format), we need to get the user and return their email
            // This is not ideal but helps with migration
            Err(AuthServiceError::InvalidToken)
        }
    }
}
