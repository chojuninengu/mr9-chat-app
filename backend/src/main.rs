use axum::{
    routing::{get, post},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    content: String,
}

// Shared state for messages
struct AppState {
    messages: Mutex<Vec<Message>>,
}

#[tokio::main]
async fn main() {
    let shared_state = Arc::new(AppState {
        messages: Mutex::new(vec![
            Message { content: "Hello from Rust!".to_string() },
            Message { content: "This is a test message.".to_string() },
            Message { content: "Chat is working!".to_string() },
        ]),
    });

    let app = Router::new()
        .route("/messages", get(get_messages).post(post_message))
        .layer(CorsLayer::permissive())
        .with_state(shared_state);

    let listener = TcpListener::bind("0.0.0.0:8000").await.unwrap();
    println!("🚀 Server running on http://localhost:8000");
    axum::serve(listener, app).await.unwrap();
}

// Get all messages
async fn get_messages(state: axum::extract::State<Arc<AppState>>) -> impl IntoResponse {
    let messages = state.messages.lock().unwrap();
    Json(messages.clone())
}

// Post a new message
async fn post_message(
    state: axum::extract::State<Arc<AppState>>,
    Json(new_message): Json<Message>,
) -> impl IntoResponse {
    let mut messages = state.messages.lock().unwrap();
    messages.push(new_message);
    (StatusCode::CREATED, Json(messages.clone()))
}
