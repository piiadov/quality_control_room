//! Virtual Engineer - AI Chat Service for Quality Control Room

mod chat;
mod ollama;

use axum::{routing::get, Router};
use chat::{ws_handler, AppState};
use ollama::OllamaClient;
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Documentation embedded at compile time
const HELP_CONTENT: &str = include_str!("../../ui/public/help.md");
const THEORY_CONTENT: &str = include_str!("../../models_gen/THEORY.md");
const TECH_DOC_CONTENT: &str = include_str!("../../doc/tech_doc.md");

/// Default LLM model
const DEFAULT_MODEL: &str = "phi3:mini";

fn build_documentation() -> String {
    format!(
        r#"# USER GUIDE
{}

# THEORETICAL FOUNDATION
{}

# TECHNICAL DOCUMENTATION
{}"#,
        HELP_CONTENT, THEORY_CONTENT, TECH_DOC_CONTENT
    )
}

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "virtual_engineer=info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Get model from env or use default
    let model = std::env::var("ENGINEER_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.to_string());
    info!("Using LLM model: {}", model);

    // Create Ollama client with all documentation as system context
    let documentation = build_documentation();
    info!("Loaded {} bytes of documentation", documentation.len());
    let ollama = OllamaClient::new(&model, &documentation);
    
    // Check Ollama availability
    if ollama.health_check().await {
        info!("Ollama is available");
    } else {
        error!("Warning: Ollama is not available at http://127.0.0.1:11434");
        error!("Install Ollama and run: ollama pull {}", model);
    }

    let state = Arc::new(AppState { ollama });

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build router
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/health", get(|| async { "OK" }))
        .layer(cors)
        .with_state(state);

    // Check for TLS certificates
    let cert_path = PathBuf::from("/etc/ssl/certs/quality-control.io-fullchain.crt");
    let key_path = PathBuf::from("/etc/ssl/private/quality-control.io.key");

    let port: u16 = std::env::var("ENGINEER_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8082);

    if cert_path.exists() && key_path.exists() {
        // TLS mode
        info!("Starting Virtual Engineer with TLS on port {}", port);
        
        let config = axum_server::tls_rustls::RustlsConfig::from_pem_file(&cert_path, &key_path)
            .await
            .expect("Failed to load TLS certificates");

        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        axum_server::bind_rustls(addr, config)
            .serve(app.into_make_service())
            .await
            .expect("Server failed");
    } else {
        // Plain HTTP mode (for development)
        info!("Starting Virtual Engineer (no TLS) on port {}", port);
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    }
}
