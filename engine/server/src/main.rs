use futures::{SinkExt, StreamExt};
use libserver::api::{handle_about, handle_calc, handle_update_bins, ApiRequest};
use warp::{Filter, ws};

#[tokio::main]
async fn main() {
    // Server IP
    let server_addr = ([0, 0, 0, 0], 8081);
    
    println!("Starting Quality Control Room Backend Server");
    println!("Listening on: {}:{}", server_addr.0.map(|x| x.to_string()).join("."), server_addr.1);
    println!("WebSocket path: wss://quality-control.io:8081/quality");
    
    // WebSocket route
    let ws_route = warp::path("quality")
        .and(warp::ws())
        .map(|ws: ws::Ws| {
            println!("WebSocket upgrade request received");
            ws.on_upgrade(handle_socket)
        });

    warp::serve(ws_route)
        .tls()
        .cert_path("/etc/letsencrypt/live/quality-control.io/fullchain.pem")
        .key_path("/etc/letsencrypt/live/quality-control.io/privkey.pem")
        .run(server_addr)
        .await;
}

async fn handle_socket(websocket: ws::WebSocket) {
    println!("New WebSocket connection established!");
    let (mut tx, mut rx) = websocket.split();

    while let Some(result) = rx.next().await {
        match result {
            Ok(msg) if msg.is_text() => {
                let text = msg.to_str().unwrap_or("");
                println!("Received message: {}", text);
                if let Ok(request) = serde_json::from_str::<ApiRequest>(text) {
                    println!("Processing command: {}", request.command);
                    match request.command.as_str() {
                        "calc" => {
                            let response = handle_calc(
                                request.kind,
                                request.test_mode,
                                request.data.unwrap_or(vec![]),
                                request.min_value.unwrap_or(f64::NAN),
                                request.max_value.unwrap_or(f64::NAN),
                                request.population_size.unwrap_or(0),
                                request.bins_number.unwrap_or(10)
                            );
                            let response_json =
                                    serde_json::to_string(&response).unwrap_or("{}".to_string());
                            if let Err(e) =
                                    tx.send(ws::Message::text(response_json)).await {
                                eprintln!("Failed to send response: {}", e);
                            }
                        }
                        "update_bins" => {
                            let response = handle_update_bins(
                                request.kind,
                                request.data.unwrap_or(vec![]),
                                request.min_value.unwrap_or(f64::NAN),
                                request.max_value.unwrap_or(f64::NAN),
                                request.bins_number.unwrap_or(10),
                                request.params_min.unwrap_or([f64::NAN;2]),
                                request.params_max.unwrap_or([f64::NAN;2]),
                                request.predicted_params.unwrap_or([f64::NAN;2]),
                                request.test_mode_params.unwrap_or([f64::NAN;2]),
                                request.test_mode
                            );
                            let response_json =
                                    serde_json::to_string(&response).unwrap_or("{}".to_string());
                            if let Err(e) =
                                    tx.send(ws::Message::text(response_json)).await {
                                eprintln!("Failed to send response: {}", e);
                            }
                        }
                        "about" => {
                            let response = handle_about();
                            let response_json =
                                serde_json::to_string(&response).unwrap_or("{}".to_string());
                            if let Err(e) = tx.send(ws::Message::text(response_json)).await {
                                eprintln!("Failed to send response: {}", e);
                            }
                        }
                        _ => {
                            let error_message = "Unsupported command";
                            if let Err(e) = tx.send(ws::Message::text(error_message)).await {
                                eprintln!("Failed to send error message: {}", e);
                            }
                        }
                    }
                } else {
                    let error_message = "Invalid request format";
                    if let Err(e) = tx.send(ws::Message::text(error_message)).await {
                        eprintln!("Failed to send error message: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }
}
