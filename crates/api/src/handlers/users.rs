use crate::auth::AuthenticatedUser;
use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::{types::AttributeValue, Client as DynamoClient};
use axum::{http::StatusCode, Json};
use lambda_http::Error;
use shared::User;
use tokio::sync::OnceCell as AsyncOnceCell;

lazy_static::lazy_static! {
    static ref TABLE_NAME: String = std::env::var("USERS_TABLE")
        .unwrap_or_else(|_| "checkmate-dev-users-table".to_string());
}

static DYNAMO_CLIENT: AsyncOnceCell<DynamoClient> = AsyncOnceCell::const_new();

async fn get_dynamo_client() -> Result<&'static DynamoClient, Error> {
    DYNAMO_CLIENT
        .get_or_init(|| async {
            let config = aws_config::defaults(BehaviorVersion::latest()).load().await;
            DynamoClient::new(&config)
        })
        .await;
    Ok(DYNAMO_CLIENT.get().unwrap())
}

#[tracing::instrument(skip(auth_user))]
pub async fn get_me(
    auth_user: AuthenticatedUser,
) -> Result<Json<User>, (StatusCode, Json<serde_json::Value>)> {
    let dynamo_client = match get_dynamo_client().await {
        Ok(client) => client,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    };

    let response = match dynamo_client
        .get_item()
        .table_name(TABLE_NAME.as_str())
        .key("user_id", AttributeValue::S(auth_user.claims.sub.clone()))
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Failed to get item: {:?}", e)})),
            ))
        }
    };

    if let Some(item) = response.item {
        let user: User = match serde_dynamo::from_item(item) {
            Ok(u) => u,
            Err(e) => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": format!("Deserialization error: {:?}", e)})),
                ))
            }
        };
        Ok(Json(user))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "User not found"})),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::{AuthenticatedUser, Claims};

    #[test]
    fn test_get_me_function_compiles() {
        // This test ensures the function compiles and types check
        // For real testing, we'd need to mock DynamoDB
    }

    #[test]
    fn test_authenticated_user_creation() {
        // Test that AuthenticatedUser can be created with valid claims
        let claims = Claims {
            sub: "test-sub".to_string(),
            email: Some("test@example.com".to_string()),
            cognito_username: Some("testuser".to_string()),
            exp: 1672531200,
            iat: 1672444800,
            token_use: Some("id".to_string()),
            email_verified: Some(true),
            iss: Some("https://cognito-idp.eu-west-1.amazonaws.com/eu-west-1_TEST".to_string()),
            aud: Some("test-aud".to_string()),
            event_id: None,
            auth_time: Some(1672444800),
            jti: None,
        };
        let auth_user = AuthenticatedUser { claims };
        assert_eq!(auth_user.claims.sub, "test-sub");
        assert_eq!(auth_user.claims.email, Some("test@example.com".to_string()));
    }

    #[test]
    fn test_table_name_env_var() {
        // Test that TABLE_NAME uses env var or default
        // Since it's lazy_static, we can't easily test, but ensure it's defined
        assert!(!TABLE_NAME.is_empty());
    }
}
