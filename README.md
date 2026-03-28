# axum-hello-ws

Two WebSocket echo servers deployed to Cloudflare Containers, demonstrating different approaches to handling WebSocket upgrades.

## Binaries

**`axum`** — WebSocket server built with [axum](https://github.com/tokio-rs/axum). Routes: `GET /` returns `Hello, World!`, `WS /ws` echoes messages with a `hello: ` prefix.

**`hyper-ws`** — Same behaviour, implemented directly with raw hyper + tokio-tungstenite (`WebSocketStream::from_raw_socket`), without any framework. Useful as a reference for how the HTTP upgrade must be handled for CF Containers to work.

## Why hyper is required alongside tokio-tungstenite for CF Containers

CF Containers acts as an HTTP reverse proxy to the container. When a WebSocket upgrade arrives, CF's HTTP layer consumes the upgrade request and expects the container to respond with `101 Switching Protocols` as a proper HTTP response within the normal request/response cycle.

tokio-tungstenite's `accept_async(stream)` bypasses this entirely — it tries to read and parse the HTTP upgrade request directly from the raw TCP stream. But CF has already consumed those bytes; the stream the container sees contains no HTTP headers. `accept_async` hangs waiting for data that will never arrive.

The fix is to use hyper as the HTTP server layer:

1. `http1::Builder::new().serve_connection(io, service).with_upgrades()` — hyper handles the HTTP request from CF
2. The service handler returns `101 Switching Protocols` as an HTTP response immediately
3. `hyper::upgrade::on(&mut req).await` hands back the raw socket once CF acknowledges the upgrade
4. `WebSocketStream::from_raw_socket(upgraded, Role::Server, None)` wraps it — no second HTTP handshake

This is exactly what axum does internally in `WebSocketUpgrade::on_upgrade`.

## Local dev

```sh
npm install
```

### axum

```sh
npx wrangler dev
```

Test:
```sh
curl http://localhost:8787/
websocat ws://localhost:8787/ws
```

### hyper-ws

```sh
npx wrangler dev --config wrangler.hyper-ws.toml
```

Test:
```sh
curl http://localhost:8787/
websocat ws://localhost:8787/ws
```
