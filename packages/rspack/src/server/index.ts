import path from 'path';
import type * as http from 'http';
import type * as Config from '../config';
import type { FSWatcher } from 'chokidar';
import { createWebSocketServer, WebSocketServer } from './ws';
import { closeHttpServer, createHttpServer } from './http';
import { openBrowser } from './open';
import { createWatcher } from './watch';
import { createMiddleware } from './app';
import type { MiddlewareServer } from './app';

export interface Server {
  http: http.Server;
  ws: WebSocketServer;
  watcher: FSWatcher;
  connect: MiddlewareServer;
  config: ResolvedDevConfig;
  start(): Promise<void>;
  close(): Promise<void>;
}

export interface ResolvedDevConfig extends Config.RspackOptions {
  dev: Required<Config.Dev>
}

function resolveDevConfig(config: Config.RspackOptions): ResolvedDevConfig {
  const devConfig = config.dev ?? {};
  const port = devConfig.port ?? 8081;
  const hmr = devConfig.hmr ?? true;
  const open = devConfig.open ?? true;
  const directory = devConfig.static?.directory ?? path.resolve(config.context, "public")
  return {
    ...config,
    dev: {
      port,
      hmr,
      open,
      static: {
        directory
      }
    }
  }
}

function createServer(userConfig: Config.RspackOptions = {}): Server {
  const config = resolveDevConfig(userConfig);
  const app = createMiddleware(config);
  const http = createHttpServer(app);
  const ws = createWebSocketServer();
  const watcher = createWatcher(config);

  http.on('upgrade', (req, socket, head) => {
    if (req.headers['sec-websocket-protocol'] !== "web-server") {
      return;
    }
    ws.server.handleUpgrade(req, socket, head, client => {
      ws.server.emit('connection', client, req)
    })
  })


  const server = {
    watcher,
    connect: app,
    http,
    ws,
    config,
    async start() {
      const protocol = 'http';
      const hostname = "localhost";
      const port = config.dev.port;

      http.listen(port, hostname, () => {
        console.log('start http server');
      })

      if (config.dev.port) {
        const url = `${protocol}://${hostname}:${port}`;
        await openBrowser(url)
      }
    },
    async close() {
      watcher.close();
      ws.server.close();
      closeHttpServer(http)
    }
  }

  return server;
}

export {
  createServer
}