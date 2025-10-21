use futures::{SinkExt, StreamExt};
use serde_json::json;
use std::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting WebSocket manual test...");

    // WebSocket URL from deployment
    let websocket_url = "wss://yphq15v1gk.execute-api.eu-west-1.amazonaws.com/dev";

    println!("Connecting to: {}", websocket_url);

    // Connect to WebSocket
    let (mut ws_stream, _) = connect_async(websocket_url).await?;
    println!("✅ WebSocket connection established!");

    // Test 1: Send ping message
    println!("\n--- Test 1: Ping-Pong ---");
    let ping_message = json!({
        "action": "ping"
    });

    println!("Sending ping: {}", ping_message);
    ws_stream.send(Message::Text(ping_message.to_string())).await?;

    // Wait for response
    tokio::time::sleep(Duration::from_secs(2)).await;

    if let Some(message) = ws_stream.next().await {
        match message {
            Ok(msg) => {
                println!("✅ Received response: {}", msg);
                if let Ok(response_json) = serde_json::from_str::<serde_json::Value>(&msg.to_string()) {
                    println!("Parsed response: {:#}", response_json);
                }
            }
            Err(e) => println!("❌ Error receiving message: {}", e),
        }
    } else {
        println!("❌ No response received for ping");
    }

    // Test 2: Get connection status
    println!("\n--- Test 2: Connection Status ---");
    let status_message = json!({
        "action": "get_connection_status"
    });

    println!("Sending status request: {}", status_message);
    ws_stream.send(Message::Text(status_message.to_string())).await?;

    // Wait for response
    tokio::time::sleep(Duration::from_secs(2)).await;

    if let Some(message) = ws_stream.next().await {
        match message {
            Ok(msg) => {
                println!("✅ Received status: {}", msg);
                if let Ok(response_json) = serde_json::from_str::<serde_json::Value>(&msg.to_string()) {
                    println!("Parsed status: {:#}", response_json);
                }
            }
            Err(e) => println!("❌ Error receiving status: {}", e),
        }
    } else {
        println!("❌ No response received for status");
    }

    // Test 3: Echo test
    println!("\n--- Test 3: Echo Test ---");
    let echo_message = json!({
        "action": "echo",
        "data": "Hello WebSocket!"
    });

    println!("Sending echo: {}", echo_message);
    ws_stream.send(Message::Text(echo_message.to_string())).await?;

    // Wait for response
    tokio::time::sleep(Duration::from_secs(2)).await;

    if let Some(message) = ws_stream.next().await {
        match message {
            Ok(msg) => {
                println!("✅ Received echo: {}", msg);
                if let Ok(response_json) = serde_json::from_str::<serde_json::Value>(&msg.to_string()) {
                    println!("Parsed echo: {:#}", response_json);
                }
            }
            Err(e) => println!("❌ Error receiving echo: {}", e),
        }
    } else {
        println!("❌ No response received for echo");
    }

    // Test 4: Invalid action
    println!("\n--- Test 4: Invalid Action ---");
    let invalid_message = json!({
        "action": "invalid_action"
    });

    println!("Sending invalid action: {}", invalid_message);
    ws_stream.send(Message::Text(invalid_message.to_string())).await?;

    // Wait for response
    tokio::time::sleep(Duration::from_secs(2)).await;

    if let Some(message) = ws_stream.next().await {
        match message {
            Ok(msg) => {
                println!("✅ Received error response: {}", msg);
                if let Ok(response_json) = serde_json::from_str::<serde_json::Value>(&msg.to_string()) {
                    println!("Parsed error: {:#}", response_json);
                }
            }
            Err(e) => println!("❌ Error receiving error response: {}", e),
        }
    } else {
        println!("❌ No response received for invalid action");
    }

    // Close connection
    println!("\n--- Closing Connection ---");
    ws_stream.close(None).await?;
    println!("✅ WebSocket connection closed!");

    println!("\n🎉 WebSocket manual test completed!");
    Ok(())
}
