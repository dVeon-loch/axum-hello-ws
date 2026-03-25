import { Container, getContainer } from "@cloudflare/containers";

export interface Env {
  CONTAINER: DurableObjectNamespace;
}

export class AxumHelloContainer extends Container {
  defaultPort = 8080;
  sleepAfter = "10m";
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    // cf-container-target-port bypasses JSRPC limitations — stub.fetch() via JSRPC
    // cannot forward WebSockets; this header lets CF route directly to the container port.
    const url = new URL(request.url);
    const headers = new Headers(request.headers);

    // Route based on both path and whether this is a WebSocket upgrade.
    // axum serves GET / and WS /ws on the same port 8080, so all routes go there.
    const isWs = request.headers.get("Upgrade")?.toLowerCase() === "websocket";
    if (isWs && url.pathname === "/ws") {
      // WebSocket echo handler
      headers.set("cf-container-target-port", "8080");
    } else if (!isWs && url.pathname === "/") {
      // Plain HTTP hello endpoint
      headers.set("cf-container-target-port", "8080");
    } else {
      headers.set("cf-container-target-port", "8080");
    }

    const internalRequest = new Request(request, { headers });
    return await getContainer(env.CONTAINER, "primary").fetch(internalRequest);
  },
};
