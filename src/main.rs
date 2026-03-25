use axum::{
    Router,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(hello))
        .route("/ws", get(ws_handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("Listening on http://0.0.0.0:8080");
    axum::serve(listener, app).await.unwrap();
}

async fn hello() -> &'static str {
    "Hello, World!"
}

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    while let Some(Ok(msg)) = socket.recv().await {
        if let Message::Text(text) = msg {
            let response = format!("hello: {text}");
            if socket.send(Message::Text(response.into())).await.is_err() {
                break;
            }
        }
    }
}
