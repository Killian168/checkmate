use crate::models::user::User;
use crate::repositories::errors::user_repository_errors::UserRepositoryError;
use async_trait::async_trait;
use aws_sdk_dynamodb::Client;
use serde_dynamo::{from_item, to_attribute_value, to_item};

pub struct DynamoDbUserRepository {
    pub client: Client,
    pub table_name: String,
}

impl DynamoDbUserRepository {
    pub fn new(client: Client) -> Self {
        let table_name =
            std::env::var("USERS_TABLE").expect(&"USERS_TABLE environment variable must be set");
        Self { client, table_name }
    }
}

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create_user(&self, user: &User) -> Result<(), UserRepositoryError>;
    async fn get_user_by_id(&self, user_id: &str) -> Result<User, UserRepositoryError>;
    async fn get_user_by_email(&self, email: &str) -> Result<User, UserRepositoryError>;
    async fn update_user(&self, user: &User) -> Result<(), UserRepositoryError>;
    async fn delete_user(&self, user_id: &str) -> Result<(), UserRepositoryError>;
    async fn email_exists(&self, email: &str) -> Result<bool, UserRepositoryError>;
}

#[async_trait]
impl UserRepository for DynamoDbUserRepository {
    async fn create_user(&self, user: &User) -> Result<(), UserRepositoryError> {
        let item = to_item(user).map_err(|e| UserRepositoryError::Serialization(e.to_string()))?;
        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await
            .map_err(|e| UserRepositoryError::DynamoDb(e.to_string()))?;
        Ok(())
    }

    async fn get_user_by_id(&self, user_id: &str) -> Result<User, UserRepositoryError> {
        let output = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key(
                "id",
                to_attribute_value(user_id)
                    .map_err(|e| UserRepositoryError::Serialization(e.to_string()))?,
            )
            .send()
            .await
            .map_err(|e| UserRepositoryError::DynamoDb(e.to_string()))?;
        if let Some(item) = output.item {
            let user: User =
                from_item(item).map_err(|e| UserRepositoryError::Serialization(e.to_string()))?;
            Ok(user)
        } else {
            Err(UserRepositoryError::NotFound)
        }
    }

    async fn get_user_by_email(&self, email: &str) -> Result<User, UserRepositoryError> {
        let result = self
            .client
            .query()
            .table_name(&self.table_name)
            .index_name("GSI_UserByEmail")
            .key_condition_expression("email = :email")
            .expression_attribute_values(
                ":email",
                to_attribute_value(email)
                    .map_err(|e| UserRepositoryError::Serialization(e.to_string()))?,
            )
            .send()
            .await;
        match result {
            Ok(output) => {
                if let Some(items) = output.items {
                    if let Some(item) = items.into_iter().next() {
                        let user = from_item(item)
                            .map_err(|e| UserRepositoryError::Serialization(e.to_string()))?;
                        Ok(user)
                    } else {
                        Err(UserRepositoryError::NotFound)
                    }
                } else {
                    Err(UserRepositoryError::NotFound)
                }
            }
            Err(e) => {
                let error_str = e.to_string();
                if error_str.contains("ResourceNotFoundException")
                    || error_str.contains("ValidationException")
                {
                    return Err(UserRepositoryError::DynamoDb("User email index not available. Please ensure the GSI 'GSI_UserByEmail' exists and is active.".to_string()));
                }
                Err(UserRepositoryError::DynamoDb(error_str))
            }
        }
    }

    async fn update_user(&self, user: &User) -> Result<(), UserRepositoryError> {
        let item = to_item(user).map_err(|e| UserRepositoryError::Serialization(e.to_string()))?;
        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await
            .map_err(|e| UserRepositoryError::DynamoDb(e.to_string()))?;
        Ok(())
    }

    async fn delete_user(&self, user_id: &str) -> Result<(), UserRepositoryError> {
        let result = self
            .client
            .delete_item()
            .table_name(&self.table_name)
            .key(
                "id",
                to_attribute_value(user_id)
                    .map_err(|e| UserRepositoryError::Serialization(e.to_string()))?,
            )
            .condition_expression("attribute_exists(id)")
            .send()
            .await;
        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                let error_str = e.to_string();
                if error_str.contains("ConditionalCheckFailedException") {
                    return Err(UserRepositoryError::NotFound);
                }
                Err(UserRepositoryError::DynamoDb(error_str))
            }
        }
    }

    async fn email_exists(&self, email: &str) -> Result<bool, UserRepositoryError> {
        let result = self
            .client
            .query()
            .table_name(&self.table_name)
            .index_name("GSI_UserByEmail")
            .key_condition_expression("email = :email")
            .expression_attribute_values(
                ":email",
                to_attribute_value(email)
                    .map_err(|e| UserRepositoryError::Serialization(e.to_string()))?,
            )
            .limit(1)
            .send()
            .await;
        match result {
            Ok(output) => {
                let exists = output
                    .items
                    .as_ref()
                    .map_or(false, |items| !items.is_empty());
                Ok(exists)
            }
            Err(e) => {
                let error_str = e.to_string();
                if error_str.contains("ResourceNotFoundException")
                    || error_str.contains("ValidationException")
                {
                    Ok(false)
                } else {
                    Err(UserRepositoryError::DynamoDb(error_str))
                }
            }
        }
    }
}
