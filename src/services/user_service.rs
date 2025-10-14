use std::sync::Arc;

use crate::models::user::{User, UserRepository, UserRepositoryError};
use crate::services::errors::user_service_errors::UserServiceError;

pub struct UserService {
    repository: Arc<dyn UserRepository + Send + Sync>,
}

impl UserService {
    pub fn new(repository: Arc<dyn UserRepository + Send + Sync>) -> Self {
        UserService { repository }
    }

    pub async fn create_user(
        &self,
        email: &str,
        password: &str,
        first_name: &str,
        last_name: &str,
    ) -> Result<User, UserServiceError> {
        if email.is_empty() || password.is_empty() || first_name.is_empty() || last_name.is_empty()
        {
            return Err(UserServiceError::ValidationError(
                "Email, password, first name, or last name cannot be empty".to_string(),
            ));
        }
        if self
            .repository
            .email_exists(email)
            .await
            .map_err(|e| UserServiceError::RepositoryError(e.to_string()))?
        {
            return Err(UserServiceError::UserAlreadyExists);
        }
        let hashed_password = password.to_string(); // Replace with real hashing
        let user = User::new(
            email.to_string(),
            hashed_password,
            first_name.to_string(),
            last_name.to_string(),
        );
        self.repository
            .create_user(&user)
            .await
            .map_err(|e| UserServiceError::RepositoryError(e.to_string()))?;
        Ok(user)
    }

    pub async fn get_user_by_id(&self, user_id: &str) -> Result<User, UserServiceError> {
        if user_id.is_empty() {
            return Err(UserServiceError::ValidationError(
                "User ID cannot be empty".to_string(),
            ));
        }
        self.repository
            .get_user_by_id(user_id)
            .await
            .map_err(|e| match e {
                UserRepositoryError::NotFound => UserServiceError::UserNotFound,
                _ => UserServiceError::RepositoryError(e.to_string()),
            })
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<User, UserServiceError> {
        if email.is_empty() {
            return Err(UserServiceError::ValidationError(
                "Email cannot be empty".to_string(),
            ));
        }
        self.repository
            .get_user_by_email(email)
            .await
            .map_err(|e| match e {
                UserRepositoryError::NotFound => UserServiceError::UserNotFound,
                _ => UserServiceError::RepositoryError(e.to_string()),
            })
    }

    pub async fn delete_user(&self, user_id: &str) -> Result<(), UserServiceError> {
        if user_id.is_empty() {
            return Err(UserServiceError::ValidationError(
                "User ID cannot be empty".to_string(),
            ));
        }
        self.repository
            .delete_user(user_id)
            .await
            .map_err(|e| match e {
                UserRepositoryError::NotFound => UserServiceError::UserNotFound,
                _ => UserServiceError::RepositoryError(e.to_string()),
            })
    }
}
