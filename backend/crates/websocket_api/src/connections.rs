use aws_sdk_dynamodb::types::AttributeValue;
use lambda_runtime::Error;
use serde_dynamo;
use std::collections::HashMap;
use tracing::info;

use crate::models::Connection;
use crate::AppState;

pub async fn store_connection(state: &AppState, connection: &Connection) -> Result<(), Error> {
    info!(
        "Storing connection for user {} with connection_id {}",
        connection.user_id, connection.connection_id
    );
    let item = serde_dynamo::to_item(connection)?;
    state
        .dynamodb
        .put_item()
        .table_name(&state.connections_table)
        .set_item(Some(item))
        .send()
        .await?;
    info!(
        "Successfully stored connection for user {} with connection_id {}",
        connection.user_id, connection.connection_id
    );
    Ok(())
}

pub async fn get_user_id_by_connection(
    state: &AppState,
    connection_id: &str,
) -> Result<Option<String>, Error> {
    info!("Looking up user_id for connection_id {}", connection_id);
    let key = HashMap::from([(
        "connection_id".to_string(),
        AttributeValue::S(connection_id.to_string()),
    )]);
    let resp = state
        .dynamodb
        .get_item()
        .table_name(&state.connections_table)
        .set_key(Some(key))
        .send()
        .await?;
    if let Some(item) = resp.item {
        let connection: Connection = serde_dynamo::from_item(item)?;
        info!(
            "Found user_id {} for connection_id {}",
            connection.user_id, connection_id
        );
        Ok(Some(connection.user_id))
    } else {
        info!("No user found for connection_id {}", connection_id);
        Ok(None)
    }
}

pub async fn remove_connection(state: &AppState, connection_id: &str) -> Result<(), Error> {
    info!("Removing connection for connection_id {}", connection_id);
    let key = HashMap::from([(
        "connection_id".to_string(),
        AttributeValue::S(connection_id.to_string()),
    )]);
    state
        .dynamodb
        .delete_item()
        .table_name(&state.connections_table)
        .set_key(Some(key))
        .send()
        .await?;
    info!(
        "Successfully removed connection for connection_id {}",
        connection_id
    );
    Ok(())
}
