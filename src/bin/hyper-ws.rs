use std::{convert::Infallible, net::SocketAddr};

use http_body_util::Full;
use hyper::{
    body::{Bytes, Incoming},
    header::{CONNECTION, SEC_WEBSOCKET_ACCEPT, SEC_WEBSOCKET_KEY, UPGRADE},
    server::conn::http1,
    service::service_fn,
    upgrade::Upgraded,
    Method, Request, Response, StatusCode,
};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use tokio_tungstenite::{
    tungstenite::{handshake::derive_accept_key, protocol::Role, Message},
    WebSocketStream,
};
use futures_util::{SinkExt, StreamExt};

type Body = Full<Bytes>;

async fn handle_socket(mut ws: WebSocketStream<TokioIo<Upgraded>>) {
    while let Some(Ok(msg)) = ws.next().await {
        if msg.is_text() || msg.is_binary() {
            let reply = Message::text(format!("hello: {}", msg.to_text().unwrap_or("")));
            if ws.send(reply).await.is_err() {
                break;
            }
        }
    }
}

async fn handle_request(
    mut req: Request<Incoming>,
    _addr: SocketAddr,
) -> Result<Response<Body>, Infallible> {
    let key = req.headers().get(SEC_WEBSOCKET_KEY);
    let derived = key.map(|k| derive_accept_key(k.as_bytes()));

    if req.method() != Method::GET
        || !req
            .headers()
            .get(UPGRADE)
            .and_then(|h| h.to_str().ok())
            .map(|h| h.eq_ignore_ascii_case("websocket"))
            .unwrap_or(false)
        || key.is_none()
    {
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .body(Body::from("Hello, World!"))
            .unwrap());
    }

    tokio::spawn(async move {
        match hyper::upgrade::on(&mut req).await {
            Ok(upgraded) => {
                let ws = WebSocketStream::from_raw_socket(
                    TokioIo::new(upgraded),
                    Role::Server,
                    None,
                )
                .await;
                handle_socket(ws).await;
            }
            Err(e) => eprintln!("upgrade error: {e}"),
        }
    });

    let mut res = Response::new(Body::default());
    *res.status_mut() = StatusCode::SWITCHING_PROTOCOLS;
    res.headers_mut().append(CONNECTION, "upgrade".parse().unwrap());
    res.headers_mut().append(UPGRADE, "websocket".parse().unwrap());
    res.headers_mut()
        .append(SEC_WEBSOCKET_ACCEPT, derived.unwrap().parse().unwrap());
    Ok(res)
}

#[tokio::main]
async fn main() {
    let port = std::env::args()
        .nth(1)
        .or_else(|| std::env::var("PORT").ok())
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8080);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("Listening on http://{addr}");

    loop {
        let (stream, peer) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            let io = TokioIo::new(stream);
            let service = service_fn(move |req| handle_request(req, peer));
            if let Err(e) = http1::Builder::new()
                .serve_connection(io, service)
                .with_upgrades()
                .await
            {
                eprintln!("connection error: {e}");
            }
        });
    }
}
