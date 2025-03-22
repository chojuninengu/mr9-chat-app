use axum::{
    extract::{Extension, Json, WebSocketUpgrade, ws::{Message, WebSocket}},
    routing::{post, get},
    response::IntoResponse,
    Router,
};
use serde::{Deserialize, Serialize};
use std::{sync::Arc};
use tokio::sync::{Mutex, broadcast};
use tokio::net::TcpListener;
use tower_http::cors::{CorsLayer, Any};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ChatMessage {
    username: String,
    content: String,
}

type SharedMessages = Arc<Mutex<Vec<ChatMessage>>>;

#[tokio::main]
async fn main() {
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

// 📨 POST /send - Send a new message
async fn send_message(
    Extension(messages): Extension<SharedMessages>,
    Extension(tx): Extension<broadcast::Sender<ChatMessage>>,
    Json(msg): Json<ChatMessage>,
) -> Json<Vec<ChatMessage>> {
    let mut stored_messages = messages.lock().await;
    stored_messages.push(msg.clone());
    let _ = tx.send(msg.clone());  // Broadcast to WebSocket clients
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
