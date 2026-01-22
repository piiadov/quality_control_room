//! Quality Control Room WebSocket Server
//!
//! A standalone WebSocket server for quality control analysis.
//!
//! # Build Modes
//!
//! - **Debug**: HTTP only (no TLS required)
//! - **Release**: HTTPS required (TLS config must be present)
//!
//! # Usage
//!
//! ```bash
//! server [config.yaml]
//! ```

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
#[cfg(not(debug_assertions))]
use axum_server::tls_rustls::RustlsConfig;
use libserver::api::{handle_request, ApiRequest, ApiResponse, AppState};
use libserver::config::Config;
use libserver::xgb;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "server=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args: Vec<String> = env::args().collect();
    let config_path = if args.len() > 1 { &args[1] } else { "config.yaml" };

    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║           Quality Control Room - WebSocket Server             ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!();

    // Show build mode
    #[cfg(debug_assertions)]
    println!("Build mode: DEBUG (HTTP only)");
    #[cfg(not(debug_assertions))]
    println!("Build mode: RELEASE (HTTPS required)");
    println!();

    // Load configuration
    println!("Loading configuration from: {}", config_path);
    let config = match Config::load(config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            std::process::exit(1);
        }
    };

    // Release mode: require TLS config
    #[cfg(not(debug_assertions))]
    if config.server.tls.is_none() {
        eprintln!("Error: TLS configuration required in release mode");
        eprintln!("Add 'tls' section to config.yaml with cert_path and key_path");
        std::process::exit(1);
    }

    // Initialize XGBoost wrapper
    println!("Initializing XGBoost wrapper...");
    if let Err(e) = xgb::init() {
        eprintln!("Failed to initialize xgbwrapper: {}", e);
        std::process::exit(1);
    }

    // Create shared state
    let state = Arc::new(AppState::new(config.clone()));

    println!();
    println!("Server configuration:");
    println!("  Host: {}", config.server.host);
    println!("  Port: {}", config.server.port);
    println!("  WebSocket path: /{}", config.server.ws_path);
    println!("  Models directory: {}", config.models.models_dir);
    println!("  Available sample sizes: {:?}", config.models.sample_sizes);

    // Build router with WebSocket route
    let ws_path = format!("/{}", config.server.ws_path);
    let app = Router::new()
        .route(&ws_path, get(ws_handler))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));

    println!();

    // Debug mode: HTTP only
    #[cfg(debug_assertions)]
    {
        if config.server.tls.is_some() {
            println!("Note: TLS config ignored in debug mode");
        }
        println!("Starting HTTP/WS server (debug mode)...");
        println!(
            "  URL: ws://{}:{}/{}",
            config.server.host, config.server.port, config.server.ws_path
        );

        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        if let Err(e) = axum::serve(listener, app).await {
            eprintln!("Server error: {}", e);
        }
    }

    // Release mode: HTTPS required
    #[cfg(not(debug_assertions))]
    {
        let tls = config.server.tls.as_ref().unwrap();
        println!("Starting HTTPS/WSS server (release mode)...");
        println!("  TLS cert: {}", tls.cert_path);
        println!("  TLS key: {}", tls.key_path);
        println!(
            "  URL: wss://{}:{}/{}",
            config.server.host, config.server.port, config.server.ws_path
        );

        let rustls_config = match RustlsConfig::from_pem_file(&tls.cert_path, &tls.key_path).await {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to load TLS config: {}", e);
                std::process::exit(1);
            }
        };

        if let Err(e) = axum_server::bind_rustls(addr, rustls_config)
            .serve(app.into_make_service())
            .await
        {
            eprintln!("Server error: {}", e);
        }
    }

    // Cleanup
    xgb::cleanup();
}

/// WebSocket upgrade handler
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_connection(socket, state))
}

/// Handle a WebSocket connection
async fn handle_connection(mut socket: WebSocket, state: Arc<AppState>) {
    tracing::info!("New WebSocket connection established");

    while let Some(result) = socket.recv().await {
        match result {
            Ok(Message::Text(text)) => {
                let response = match serde_json::from_str::<ApiRequest>(&text) {
                    Ok(req) => {
                        tracing::info!("Processing command: {}", req.command);
                        handle_request(&req, &state)
                    }
                    Err(e) => {
                        tracing::warn!("Invalid request: {}", e);
                        ApiResponse {
                            command: "error".into(),
                            success: false,
                            message: Some(format!("Invalid request format: {}", e)),
                            ..Default::default()
                        }
                    }
                };

                let response_json =
                    serde_json::to_string(&response).unwrap_or_else(|_| "{}".into());

                if let Err(e) = socket.send(Message::Text(response_json.into())).await {
                    tracing::error!("Failed to send response: {}", e);
                    break;
                }
            }
            Ok(Message::Close(_)) => {
                tracing::info!("Client disconnected");
                break;
            }
            Ok(_) => {} // Ignore other message types (binary, ping, pong)
            Err(e) => {
                tracing::error!("WebSocket error: {}", e);
                break;
            }
        }
    }

    tracing::info!("Connection closed");
}
