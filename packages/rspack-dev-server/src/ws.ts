import WebSocket from "ws";
import type { Server as HttpServer } from "http";

const INTERVAL = 1000;

export type ClientConnection = WebSocket & { isAlive?: boolean };

export interface WebSocketServer {
	clients: ClientConnection[];
	implementation: WebSocket.Server;
	stop(): Promise<void>;
}

export function createWebSocketServer(
	server: HttpServer,
	options: {}
): WebSocketServer {
	let implementation = new WebSocket.Server({
		port: 25566
	});
	let clients: ClientConnection[] = [];

	server.on("upgrade", (req, socket, head) => {
		if (!implementation.shouldHandle(req)) {
			return;
		}

		implementation.handleUpgrade(req, socket, head, connection => {
			implementation.emit("connection", connection, req);
		});
	});

	server.on("error", err => {
		console.error(err);
	});

	const interval = setInterval(() => {
		clients.forEach(client => {
			if (client.isAlive === false) {
				client.terminate();
				return;
			}
			client.isAlive = false;
			client.ping(() => {});
		});
	}, INTERVAL);

	implementation.on("connection", (client: ClientConnection) => {
		clients.push(client);
		client.isAlive = true;
		client.on("pong", () => {
			client.isAlive = true;
		});
		client.on("close", () => {
			const index = clients.indexOf(client);
			clients.splice(index, 1);
		});
	});

	implementation.on("close", () => {
		clearInterval(interval);
	});

	return {
		clients,
		implementation,
		async stop() {
			await new Promise(resolve => {
				implementation.close(() => {
					implementation = null;
					resolve({});
				});
			});
			for (const client of clients) {
				client.terminate();
			}
			clients = [];
		}
	};
}
