import connect from 'connect';
import http, { IncomingMessage, ServerResponse } from 'http';
import sirv from 'sirv';
import path from 'path';
import fs from 'fs';
import ws, { WebSocketServer } from 'ws';
import { Socket } from 'net';
import history from 'connect-history-api-fallback';
import Rspack from '../rspack';
interface DevOptions {
  root: string;
  public: string;
  bundler: Rspack
}
export class DevServer {
  #app;
  _server!: http.Server;
  _wsServer!: ws.Server;
  _webSockets: WebSocket[] = [];
  constructor(options: DevOptions) {
    const app = (this.#app = connect());
    const outdir = path.resolve(options.root, options.public);
    console.log('public:', outdir);
    app.use(history());
    if(options.bundler.options.lazyCompiler) {
      app.use(async (req, res, next) => {
        if(fs.existsSync(path.join(outdir, req.url))) {
          await options.bundler.lazyBuild(req.url.slice(1));
        }
        next();
      })
    }
    app.use(
      sirv(outdir, {
        dev: true,
      })
    );
  }
  static create(options: DevOptions) {
    const _server = new DevServer(options);
  }
  async start() {
    const server = http.createServer(this.#app).listen(4444, () => {
      console.log(`listen at: http://127.0.0.1:4444`);
    });
    this._server = server;
    await this.createWebSocketServer();
  }
  async createWebSocketServer() {
    const server = this._server;
    if (!server) {
      return;
    }
    const WebSocket = await import('ws');
    const wss = new WebSocket.WebSocketServer({
      noServer: true,
    });
    this._wsServer = wss;

    server.on('upgrade', (req, socket, head) => {
      // Only handle upgrades to Speedy Dev Server requests, ignore others.
      if (req.headers['sec-websocket-protocol'] !== 'web-server') {
        return;
      }
      wss.handleUpgrade(req, socket as Socket, head, (client) => {
        wss.emit('connection', client, req);
      });
    });

    wss.on('connection', (socket: any) => {
      this._webSockets.push(socket);
      socket.on('message', (data: any) => {
        const message = JSON.parse(data.toString());
      });
      socket.on('close', () => {
        const index = this._webSockets.indexOf(socket);
        if (index >= 0) {
          this._webSockets.splice(index, 1);
        }
      });
    });
  }
  broadcast(payload: any) {
    for (const socket of this._webSockets) {
      socket.send(JSON.stringify(payload));
    }
  }
}
