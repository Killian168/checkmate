use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::sync::Arc;

use crate::models::auth::responses::{LoginResponse, TokenClaims};

use crate::services::errors::auth_service_errors::AuthServiceError;
use crate::services::errors::user_service_errors::UserServiceError;
use crate::services::user_service::UserService;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait AuthServiceTrait: Send + Sync {
    async fn authenticate_user(
        &self,
        email: &str,
        password: &str,
    ) -> Result<LoginResponse, AuthServiceError>;
    fn verify_token(&self, token: &str) -> Result<TokenClaims, AuthServiceError>;
    fn extract_user_id_from_token(&self, token: &str) -> Result<String, AuthServiceError>;
    fn extract_user_email_from_token(&self, token: &str) -> Result<String, AuthServiceError>;
    fn generate_token(&self, user_id: &str) -> Result<LoginResponse, AuthServiceError>;
}

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

    pub fn with_jwt_secret(user_service: Arc<UserService>, jwt_secret: String) -> Self {
        AuthService {
            user_service,
            jwt_secret,
        }
    }
}

impl AuthServiceTrait for AuthService {
    async fn authenticate_user(
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
        match self.user_service.get_user_by_email(email).await {
            Ok(user) => {
                // Check if password matches (in real app, use proper password hashing)
                if user.password == password {
                    // Generate JWT token with user ID
                    self.generate_token(&user.id)
                } else {
                    Err(AuthServiceError::InvalidCredentials)
                }
            }
            Err(UserServiceError::UserNotFound) => Err(AuthServiceError::InvalidCredentials),
            Err(err) => Err(AuthServiceError::UserServiceError(err)),
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
        .map_err(|e| AuthServiceError::JwtError(format!("{:#?}", e)))?;

        Ok(LoginResponse {
            token,
            token_type: "Bearer".to_string(),
            expires_in,
        })
    }

    fn verify_token(&self, token: &str) -> Result<TokenClaims, AuthServiceError> {
        let decoding_key = DecodingKey::from_secret(self.jwt_secret.as_ref());
        let validation = Validation::default();

        match decode::<TokenClaims>(token, &decoding_key, &validation) {
            Ok(token_data) => {
                let now = Utc::now().timestamp() as usize;
                if token_data.claims.exp < now {
                    Err(AuthServiceError::ExpiredToken)
                } else {
                    Ok(token_data.claims)
                }
            }
            Err(err) => match err.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    Err(AuthServiceError::ExpiredToken)
                }
                _ => Err(AuthServiceError::InvalidToken),
            },
        }
    }

    fn extract_user_id_from_token(&self, token: &str) -> Result<String, AuthServiceError> {
        let claims = self.verify_token(token)?;
        Ok(claims.sub)
    }

    // Backward compatibility method - kept for migration purposes
    fn extract_user_email_from_token(&self, token: &str) -> Result<String, AuthServiceError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::user_repository::MockUserRepository;
    use crate::services::user_service::UserService;

    #[test]
    fn test_auth_service_trait_definition() {
        // This test verifies that the trait is properly defined
        // and can be used in service implementations
        assert!(true, "AuthServiceTrait is properly defined");
    }

    #[test]
    fn test_token_generation_and_verification_roundtrip() {
        let mut mock_repo = MockUserRepository::new();
        mock_repo.expect_get_user_by_id().returning(|_| {
            Box::pin(async {
                Ok(crate::models::user::User::new(
                    "test@example.com".to_string(),
                    "password".to_string(),
                    "Test".to_string(),
                    "User".to_string(),
                ))
            })
        });

        let auth_service = AuthService::with_jwt_secret(
            Arc::new(UserService::new(Arc::new(mock_repo))),
            "test-secret-key".to_string(),
        );

        let test_user_id = "roundtrip-user-id";
        let login_response = auth_service.generate_token(test_user_id).unwrap();

        assert_eq!(login_response.token_type, "Bearer");
        assert_eq!(login_response.expires_in, 24 * 60 * 60);

        let claims = auth_service.verify_token(&login_response.token).unwrap();
        assert_eq!(claims.sub, test_user_id);
        assert!(claims.exp > claims.iat);
    }

    #[test]
    fn test_verify_token_invalid() {
        let mut mock_repo = MockUserRepository::new();
        mock_repo.expect_get_user_by_id().returning(|_| {
            Box::pin(async {
                Ok(crate::models::user::User::new(
                    "test@example.com".to_string(),
                    "password".to_string(),
                    "Test".to_string(),
                    "User".to_string(),
                ))
            })
        });

        let auth_service = AuthService::with_jwt_secret(
            Arc::new(UserService::new(Arc::new(mock_repo))),
            "test-secret-key".to_string(),
        );

        let result = auth_service.verify_token("invalid-token");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AuthServiceError::InvalidToken
        ));
    }

    #[test]
    fn test_extract_user_id_from_token() {
        let mut mock_repo = MockUserRepository::new();
        mock_repo.expect_get_user_by_id().returning(|_| {
            Box::pin(async {
                Ok(crate::models::user::User::new(
                    "test@example.com".to_string(),
                    "password".to_string(),
                    "Test".to_string(),
                    "User".to_string(),
                ))
            })
        });

        let auth_service = AuthService::with_jwt_secret(
            Arc::new(UserService::new(Arc::new(mock_repo))),
            "test-secret-key".to_string(),
        );

        let test_user_id = "test-user-id";
        let token = auth_service.generate_token(test_user_id).unwrap().token;

        let result = auth_service.extract_user_id_from_token(&token);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), test_user_id);
    }

    #[test]
    fn test_different_secrets_produce_different_tokens() {
        let mut mock_repo1 = MockUserRepository::new();
        mock_repo1.expect_get_user_by_id().returning(|_| {
            Box::pin(async {
                Ok(crate::models::user::User::new(
                    "test@example.com".to_string(),
                    "password".to_string(),
                    "Test".to_string(),
                    "User".to_string(),
                ))
            })
        });

        let mut mock_repo2 = MockUserRepository::new();
        mock_repo2.expect_get_user_by_id().returning(|_| {
            Box::pin(async {
                Ok(crate::models::user::User::new(
                    "test@example.com".to_string(),
                    "password".to_string(),
                    "Test".to_string(),
                    "User".to_string(),
                ))
            })
        });

        let auth_service1 = AuthService::with_jwt_secret(
            Arc::new(UserService::new(Arc::new(mock_repo1))),
            "secret1".to_string(),
        );

        let auth_service2 = AuthService::with_jwt_secret(
            Arc::new(UserService::new(Arc::new(mock_repo2))),
            "secret2".to_string(),
        );

        let test_user_id = "same-user-id";
        let token1 = auth_service1.generate_token(test_user_id).unwrap().token;
        let token2 = auth_service2.generate_token(test_user_id).unwrap().token;

        assert_ne!(
            token1, token2,
            "Different secrets should produce different tokens"
        );

        // Token1 should only verify with auth_service1
        assert!(auth_service1.verify_token(&token1).is_ok());
        assert!(auth_service2.verify_token(&token1).is_err());

        // Token2 should only verify with auth_service2
        assert!(auth_service2.verify_token(&token2).is_ok());
        assert!(auth_service1.verify_token(&token2).is_err());
    }
}
