import * as Connect from 'connect';
import * as Sirv from 'sirv';
import type { ResolvedDevConfig } from '.';

export type MiddlewareServer = Connect.Server;

function createStaticMiddleware(config: ResolvedDevConfig): Connect.NextHandleFunction {
  const server = Sirv.default(config.dev.static.directory, {
    dev: true
  });
  return function staticMiddleware(req, res, next) {
    server(req, res, next)
  }
}

export function createMiddleware(config: ResolvedDevConfig): Connect.Server {
  const app = Connect.default();
  app.use(createStaticMiddleware(config))
  return app;
}
