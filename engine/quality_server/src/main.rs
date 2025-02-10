use futures::{SinkExt, StreamExt};
use lib::api::{handle_about, handle_calc, ApiRequest};
use warp::Filter;

#[tokio::main]
async fn main() {
    // WebSocket route
    let ws_route = warp::path("quality")
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| ws.on_upgrade(handle_socket));

    // Start the server on port 8080
    println!("Starting WebSocket server on ws://127.0.0.1:8080/quality");
    warp::serve(ws_route).run(([127, 0, 0, 1], 8080)).await;
}

async fn handle_socket(websocket: warp::ws::WebSocket) {
    let (mut tx, mut rx) = websocket.split();

    while let Some(result) = rx.next().await {
        match result {
            Ok(msg) if msg.is_text() => {
                let text = msg.to_str().unwrap_or("");
                if let Ok(request) = serde_json::from_str::<ApiRequest>(text) {
                    match request.command.as_str() {
                        "calc" => {
                            if let Some(data) = request.data {
                                let response = handle_calc(data);
                                let response_json =
                                    serde_json::to_string(&response).unwrap_or("{}".to_string());
                                if let Err(e) =
                                    tx.send(warp::ws::Message::text(response_json)).await
                                {
                                    eprintln!("Failed to send response: {}", e);
                                }
                            } else {
                                let error_message = "Missing data for calc command";
                                if let Err(e) =
                                    tx.send(warp::ws::Message::text(error_message)).await
                                {
                                    eprintln!("Failed to send error message: {}", e);
                                }
                            }
                        }
                        "about" => {
                            let response = handle_about();
                            let response_json =
                                serde_json::to_string(&response).unwrap_or("{}".to_string());
                            if let Err(e) = tx.send(warp::ws::Message::text(response_json)).await {
                                eprintln!("Failed to send response: {}", e);
                            }
                        }
                        _ => {
                            let error_message = "Unsupported command";
                            if let Err(e) = tx.send(warp::ws::Message::text(error_message)).await {
                                eprintln!("Failed to send error message: {}", e);
                            }
                        }
                    }
                } else {
                    let error_message = "Invalid request format";
                    if let Err(e) = tx.send(warp::ws::Message::text(error_message)).await {
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
