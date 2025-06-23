use futures::{SinkExt, StreamExt};
use libserver::api::{handle_about, handle_calc, handle_update_bins, ApiRequest};
use warp::{Filter, ws};

#[tokio::main]
async fn main() {
    // Server IP
    let server_addr = ([0, 0, 0, 0], 8081);
    // WebSocket route
    let ws_route = warp::path("quality")
        .and(warp::ws())
        .map(|ws: ws::Ws| ws.on_upgrade(handle_socket));

    println!("Quality server started on ws://{}.{}.{}.{}:{}", server_addr.0[0], server_addr.0[1],
        server_addr.0[2], server_addr.0[3], server_addr.1);
    warp::serve(ws_route).run(server_addr).await;
}

async fn handle_socket(websocket: ws::WebSocket) {
    let (mut tx, mut rx) = websocket.split();

    while let Some(result) = rx.next().await {
        match result {
            Ok(msg) if msg.is_text() => {
                let text = msg.to_str().unwrap_or("");
                if let Ok(request) = serde_json::from_str::<ApiRequest>(text) {
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
