use aws_config::BehaviorVersion;
use aws_sdk_cognitoidentityprovider::Client as CognitoClient;
use serde::{Deserialize, Serialize};
use serde_dynamo;
use std::env;

use super::load_env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthTokens {
    pub id_token: String,
    pub access_token: String,
    pub refresh_token: String,
}

/// Get Cognito User Pool ID from environment
fn get_cognito_user_pool_id() -> String {
    load_env();
    env::var("COGNITO_USER_POOL_ID")
        .unwrap_or_else(|_| panic!("COGNITO_USER_POOL_ID environment variable not set."))
}

/// Get Cognito User Pool Client ID from environment
fn get_cognito_client_id() -> String {
    load_env();
    env::var("COGNITO_CLIENT_ID")
        .unwrap_or_else(|_| panic!("COGNITO_CLIENT_ID environment variable not set."))
}

/// Authenticate with AWS Cognito using username and password
pub async fn authenticate_with_cognito(email: &str, password: &str) -> Result<AuthTokens, String> {
    // Create AWS SDK config
    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let client = CognitoClient::new(&config);

    let client_id = get_cognito_client_id();

    // Initiate authentication
    let auth_response = client
        .initiate_auth()
        .auth_flow(aws_sdk_cognitoidentityprovider::types::AuthFlowType::UserPasswordAuth)
        .client_id(&client_id)
        .auth_parameters("USERNAME", email)
        .auth_parameters("PASSWORD", password)
        .send()
        .await
        .map_err(|e| format!("Failed to authenticate with Cognito: {}", e))?;

    // Handle NEW_PASSWORD_REQUIRED challenge
    if let Some(challenge_name) = auth_response.challenge_name() {
        if challenge_name
            == &aws_sdk_cognitoidentityprovider::types::ChallengeNameType::NewPasswordRequired
        {
            eprintln!("Handling NEW_PASSWORD_REQUIRED challenge...");

            let session = auth_response
                .session()
                .ok_or("No session in challenge response")?;

            // Respond to the challenge with the same password to make it permanent
            let challenge_response = client
                .respond_to_auth_challenge()
                .challenge_name(
                    aws_sdk_cognitoidentityprovider::types::ChallengeNameType::NewPasswordRequired,
                )
                .client_id(&client_id)
                .session(session)
                .challenge_responses("USERNAME", email)
                .challenge_responses("NEW_PASSWORD", password)
                .send()
                .await
                .map_err(|e| {
                    format!(
                        "Failed to respond to NEW_PASSWORD_REQUIRED challenge: {}",
                        e
                    )
                })?;

            // Extract tokens from challenge response
            let auth_result = challenge_response
                .authentication_result()
                .ok_or("No authentication result after challenge response")?;

            let id_token = auth_result
                .id_token()
                .ok_or("No ID token in challenge response")?
                .to_string();

            let access_token = auth_result
                .access_token()
                .ok_or("No access token in challenge response")?
                .to_string();

            let refresh_token = auth_result
                .refresh_token()
                .ok_or("No refresh token in challenge response")?
                .to_string();

            return Ok(AuthTokens {
                id_token,
                access_token,
                refresh_token,
            });
        } else {
            return Err(format!(
                "Unsupported authentication challenge: {:?}. Please contact administrator.",
                challenge_name
            ));
        }
    }

    // Extract tokens from the authentication result (no challenge case)
    let auth_result = auth_response
        .authentication_result()
        .ok_or("No authentication result returned. The user may need to complete a challenge or the password may be incorrect.")?;

    let id_token = auth_result
        .id_token()
        .ok_or("No ID token in authentication result")?
        .to_string();

    let access_token = auth_result
        .access_token()
        .ok_or("No access token in authentication result")?
        .to_string();

    let refresh_token = auth_result
        .refresh_token()
        .ok_or("No refresh token in authentication result")?
        .to_string();

    Ok(AuthTokens {
        id_token,
        access_token,
        refresh_token,
    })
}

/// Create a new Cognito user with verified email for testing
pub async fn create_test_cognito_user(email: &str, temp_password: &str) -> Result<(), String> {
    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let client = CognitoClient::new(&config);

    let user_pool_id = get_cognito_user_pool_id();

    // Create the user
    client
        .admin_create_user()
        .user_pool_id(&user_pool_id)
        .username(email)
        .user_attributes(
            aws_sdk_cognitoidentityprovider::types::AttributeType::builder()
                .name("email")
                .value(email)
                .build()
                .expect("Failed to build email attribute"),
        )
        .user_attributes(
            aws_sdk_cognitoidentityprovider::types::AttributeType::builder()
                .name("email_verified")
                .value("true")
                .build()
                .expect("Failed to build email_verified attribute"),
        )
        .temporary_password(temp_password)
        .message_action(aws_sdk_cognitoidentityprovider::types::MessageActionType::Suppress)
        .send()
        .await
        .map_err(|e| format!("Failed to create Cognito user: {}", e))?;

    Ok(())
}

/// Delete a Cognito user for testing cleanup
pub async fn delete_cognito_user(email: &str) -> Result<(), String> {
    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let client = CognitoClient::new(&config);

    let user_pool_id = get_cognito_user_pool_id();

    // Delete the user
    client
        .admin_delete_user()
        .user_pool_id(&user_pool_id)
        .username(email)
        .send()
        .await
        .map_err(|e| format!("Failed to delete Cognito user: {}", e))?;

    Ok(())
}

/// Create a DynamoDB user entry for testing
pub async fn create_test_dynamodb_user(user_id: &str) -> Result<(), String> {
    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let client = aws_sdk_dynamodb::Client::new(&config);

    let table_name =
        std::env::var("USERS_TABLE").unwrap_or_else(|_| "checkmate-dev-users-table".to_string());

    // Prepare user struct
    let user = shared::User {
        user_id: user_id.to_string(),
        rating: 1200,
    };

    // Serialize to DynamoDB item
    let item = serde_dynamo::to_item(user).map_err(|e| format!("Serialization error: {:?}", e))?;

    client
        .put_item()
        .table_name(&table_name)
        .set_item(Some(item))
        .send()
        .await
        .map_err(|e| format!("Failed to put item in DynamoDB: {:?}", e))?;

    Ok(())
}

pub async fn get_test_auth_token() -> String {
    let email = "tester@test.com";
    let password = "ThisisATest123!";

    let tokens = authenticate_with_cognito(email, password)
        .await
        .expect("Failed to authenticate with test credentials");

    tokens.id_token
}
