import { Container, getContainer } from "@cloudflare/containers";

export class AxumHelloContainer extends Container {
  defaultPort = 8080;
  sleepAfter = "10m";
}

export interface Env {
  CONTAINER: DurableObjectNamespace<AxumHelloContainer>;
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    return getContainer(env.CONTAINER, "singleton").fetch(request);
  },
};
