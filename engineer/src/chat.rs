//! WebSocket chat handler

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};

use crate::ollama::OllamaClient;

pub struct AppState {
    pub ollama: OllamaClient,
}

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub command: String,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub context: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub done: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available: Option<bool>,
}

impl ChatResponse {
    pub fn chunk(content: &str) -> Self {
        Self {
            command: "chunk".into(),
            content: Some(content.into()),
            done: Some(false),
            error: None,
            available: None,
        }
    }

    pub fn done() -> Self {
        Self {
            command: "done".into(),
            content: None,
            done: Some(true),
            error: None,
            available: None,
        }
    }

    pub fn error(msg: &str) -> Self {
        Self {
            command: "error".into(),
            content: None,
            done: Some(true),
            error: Some(msg.into()),
            available: None,
        }
    }

    pub fn status(available: bool) -> Self {
        Self {
            command: "status".into(),
            content: None,
            done: None,
            error: None,
            available: Some(available),
        }
    }
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    
    info!("New WebSocket connection");

    while let Some(msg) = receiver.next().await {
        let msg = match msg {
            Ok(Message::Text(text)) => text,
            Ok(Message::Close(_)) => {
                info!("Client disconnected");
                break;
            }
            Ok(_) => continue,
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
        };

        let request: ChatRequest = match serde_json::from_str(&msg) {
            Ok(r) => r,
            Err(e) => {
                let resp = ChatResponse::error(&format!("Invalid request: {}", e));
                let _ = sender.send(Message::Text(serde_json::to_string(&resp).unwrap().into())).await;
                continue;
            }
        };

        match request.command.as_str() {
            "status" => {
                let available = state.ollama.health_check().await;
                let resp = ChatResponse::status(available);
                let _ = sender.send(Message::Text(serde_json::to_string(&resp).unwrap().into())).await;
            }
            "chat" => {
                let context_str = request.context.map(|c| serde_json::to_string_pretty(&c).unwrap_or_default());
                
                match state.ollama.generate_stream(&request.message, context_str.as_deref()).await {
                    Ok(mut rx) => {
                        while let Some(chunk) = rx.recv().await {
                            let resp = ChatResponse::chunk(&chunk);
                            if sender.send(Message::Text(serde_json::to_string(&resp).unwrap().into())).await.is_err() {
                                break;
                            }
                        }
                        let resp = ChatResponse::done();
                        let _ = sender.send(Message::Text(serde_json::to_string(&resp).unwrap().into())).await;
                    }
                    Err(e) => {
                        let resp = ChatResponse::error(&e);
                        let _ = sender.send(Message::Text(serde_json::to_string(&resp).unwrap().into())).await;
                    }
                }
            }
            _ => {
                let resp = ChatResponse::error(&format!("Unknown command: {}", request.command));
                let _ = sender.send(Message::Text(serde_json::to_string(&resp).unwrap().into())).await;
            }
        }
    }
}
