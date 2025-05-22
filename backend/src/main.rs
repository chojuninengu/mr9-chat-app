use axum::{
    extract::{Extension, Json, WebSocketUpgrade, ws::{Message, WebSocket}},
    routing::{post, get},
    response::IntoResponse,
    Router,
};
use serde::{Deserialize, Serialize};
use std::{sync::Arc, env};
use tokio::sync::{Mutex, broadcast};
use tokio::net::TcpListener;
use tower_http::cors::{CorsLayer, Any};
use dotenv::dotenv;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ChatMessage {
    username: String,
    content: String,
    is_ai: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClaudeRequest {
    model: String,
    messages: Vec<ClaudeMessage>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeContent>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClaudeContent {
    text: String,
}

type SharedMessages = Arc<Mutex<Vec<ChatMessage>>>;

#[tokio::main]
async fn main() {
    dotenv().ok();
    
    let messages: SharedMessages = Arc::new(Mutex::new(Vec::new()));
    let (tx, _rx) = broadcast::channel::<ChatMessage>(100);

    let app = Router::new()
        .route("/send", post(send_message))
        .route("/messages", get(get_messages))
        .route("/ws", get(websocket_handler))
        .layer(CorsLayer::new().allow_origin(Any))
        .layer(Extension(messages))
        .layer(Extension(tx.clone()));

    let addr = "127.0.0.1:3000";
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("🚀 Server running on http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn get_ai_response(message: &str) -> Result<String, Box<dyn std::error::Error>> {
    let api_key = env::var("CLAUDE_API_KEY").expect("CLAUDE_API_KEY must be set");
    let client = reqwest::Client::new();
    
    let claude_request = ClaudeRequest {
        model: "claude-3-opus-20240229".to_string(),
        messages: vec![ClaudeMessage {
            role: "user".to_string(),
            content: message.to_string(),
        }],
    };

    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&claude_request)
        .send()
        .await?
        .json::<ClaudeResponse>()
        .await?;

    Ok(response.content[0].text.clone())
}

// 📨 POST /send - Send a new message
async fn send_message(
    Extension(messages): Extension<SharedMessages>,
    Extension(tx): Extension<broadcast::Sender<ChatMessage>>,
    Json(msg): Json<ChatMessage>,
) -> Json<Vec<ChatMessage>> {
    let mut stored_messages = messages.lock().await;
    stored_messages.push(msg.clone());
    let _ = tx.send(msg.clone());  // Broadcast to WebSocket clients

    // Get AI response if the message is from a user
    if msg.is_ai.unwrap_or(false) == false {
        if let Ok(ai_response) = get_ai_response(&msg.content).await {
            let ai_msg = ChatMessage {
                username: "Claude".to_string(),
                content: ai_response,
                is_ai: Some(true),
            };
            stored_messages.push(ai_msg.clone());
            let _ = tx.send(ai_msg);  // Broadcast AI response
        }
    }

    Json(stored_messages.clone())
}

// 📩 GET /messages - Get all messages
async fn get_messages(Extension(messages): Extension<SharedMessages>) -> Json<Vec<ChatMessage>> {
    let stored_messages = messages.lock().await;
    Json(stored_messages.clone())
}

// 🔄 WebSocket for real-time updates
async fn websocket_handler(
    ws: WebSocketUpgrade,
    Extension(tx): Extension<broadcast::Sender<ChatMessage>>,
    Extension(messages): Extension<SharedMessages>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| websocket_process(socket, tx, messages))
}

async fn websocket_process(mut socket: WebSocket, mut tx: broadcast::Sender<ChatMessage>, messages: SharedMessages) {
    let mut rx = tx.subscribe();

    // Send existing messages to new clients
    let existing_messages = messages.lock().await.clone();
    for msg in existing_messages {
        let _ = socket.send(Message::Text(serde_json::to_string(&msg).unwrap())).await;
    }

    while let Ok(msg) = rx.recv().await {
        let _ = socket.send(Message::Text(serde_json::to_string(&msg).unwrap())).await;
    }
}
