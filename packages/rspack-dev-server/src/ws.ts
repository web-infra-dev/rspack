import type { RspackDevServer } from "./server";
import WebSocket from "ws";
import WebpackWsServer from "webpack-dev-server/lib/servers/WebsocketServer";

export type ClientConnection = WebSocket & { isAlive?: boolean };

export interface WebSocketServer {
	heartbeatInterval: number;
	implementation: WebSocket.Server<WebSocket.WebSocket>;
	server: RspackDevServer;
	clients: ClientConnection[];
}

export function createWebsocketServer(
	server: RspackDevServer
): WebSocketServer {
	return new WebpackWsServer(server);
}
