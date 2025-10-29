use crate::auth::AuthenticatedUser;
use aws_sdk_dynamodb::types::AttributeValue;
use axum::{extract::Extension, http::StatusCode, Json};
use shared::User;

use crate::AppState;

#[tracing::instrument(skip(auth_user, state))]
pub async fn get_me(
    auth_user: AuthenticatedUser,
    Extension(state): Extension<AppState>,
) -> Result<Json<User>, (StatusCode, Json<serde_json::Value>)> {
    let response = match state
        .dynamo_client
        .get_item()
        .table_name(&state.users_table)
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

#[tracing::instrument(skip(auth_user, state))]
pub async fn delete_me(
    auth_user: AuthenticatedUser,
    Extension(state): Extension<AppState>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    // Delete from DynamoDB first
    if let Err(e) = state
        .dynamo_client
        .delete_item()
        .table_name(&state.users_table)
        .key("user_id", AttributeValue::S(auth_user.claims.sub.clone()))
        .send()
        .await
    {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(
                serde_json::json!({"error": format!("Failed to delete user from DynamoDB: {:?}", e)}),
            ),
        ));
    }

    // Delete from Cognito
    // Try multiple username options in case cognito_username is not set correctly
    let username_options = vec![
        auth_user.claims.cognito_username.as_deref(),
        Some(&auth_user.claims.sub),
        auth_user.claims.email.as_deref(),
    ];

    let mut last_error = None;
    for username_opt in username_options.into_iter().flatten() {
        match state
            .cognito_client
            .admin_delete_user()
            .user_pool_id(&state.cognito_user_pool_id)
            .username(username_opt)
            .send()
            .await
        {
            Ok(_) => {
                return Ok(StatusCode::NO_CONTENT);
            }
            Err(e) => {
                last_error = Some(e);
            }
        }
    }

    // If all attempts failed, return the last error
    let error = last_error.unwrap();
    return Err((
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(
            serde_json::json!({"error": format!("Failed to delete user from Cognito: {:?}", error)}),
        ),
    ));
}
