use aws_sdk_apigatewaymanagement::Client as ApiGatewayClient;
use aws_sdk_dynamodb::Client as DynamoClient;
use lambda_runtime::Error;
use serde_dynamo;
use tracing::{error, info};

use crate::models::{Connection, GameMatchedMessage};

/// Sends a game_matched notification to a player via WebSocket
pub async fn notify_player(
    api_gateway: &ApiGatewayClient,
    dynamodb: &DynamoClient,
    connections_table: &str,
    user_id: &str,
    game_id: &str,
    opponent_id: &str,
    color: &str,
    time_control: &str,
) {
    info!("Notifying player {} of new game {}", user_id, game_id);

    // Get the connection_id for this user
    let connection_id = match get_connection_id(dynamodb, connections_table, user_id).await {
        Ok(Some(conn_id)) => conn_id,
        Ok(None) => {
            error!("No active connection found for user {}", user_id);
            return;
        }
        Err(e) => {
            error!("Failed to get connection for user {}: {:?}", user_id, e);
            return;
        }
    };

    let message = GameMatchedMessage {
        action: "game_matched".to_string(),
        game_id: game_id.to_string(),
        opponent_id: opponent_id.to_string(),
        color: color.to_string(),
        time_control: time_control.to_string(),
    };

    let data = match serde_json::to_string(&message) {
        Ok(d) => d,
        Err(e) => {
            error!("Failed to serialize message: {:?}", e);
            return;
        }
    };

    match api_gateway
        .post_to_connection()
        .connection_id(&connection_id)
        .data(aws_sdk_apigatewaymanagement::primitives::Blob::new(data))
        .send()
        .await
    {
        Ok(_) => info!(
            "Successfully notified player {} of game {}",
            user_id, game_id
        ),
        Err(e) => error!("Failed to send notification to player {}: {:?}", user_id, e),
    }
}

/// Retrieves the WebSocket connection ID for a user by querying the UserIdIndex GSI
async fn get_connection_id(
    dynamodb: &DynamoClient,
    connections_table: &str,
    user_id: &str,
) -> Result<Option<String>, Error> {
    use aws_sdk_dynamodb::types::AttributeValue;

    info!(
        "Looking up connection for user {} using UserIdIndex",
        user_id
    );

    let query_result = dynamodb
        .query()
        .table_name(connections_table)
        .index_name("UserIdIndex")
        .key_condition_expression("user_id = :uid")
        .expression_attribute_values(":uid", AttributeValue::S(user_id.to_string()))
        .send()
        .await?;

    if let Some(items) = query_result.items {
        if let Some(item) = items.first() {
            let connection: Connection = serde_dynamo::from_item(item.clone())?;
            info!(
                "Found connection {} for user {}",
                connection.connection_id, user_id
            );
            return Ok(Some(connection.connection_id));
        }
    }

    info!("No connection found for user {}", user_id);
    Ok(None)
}
