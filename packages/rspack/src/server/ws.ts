import * as ws from "ws";

export type WebSocketServer = {
	// TODO: should remove
	server: ws.WebSocketServer;
	broad(payload: any): void;
};

export function createWebSocketServer(): WebSocketServer {
	let sockets: ws.WebSocket[] = [];
	const port = 23456;
	const options: ws.ServerOptions = {
		port
	};
	const wss = new ws.WebSocketServer(options);

	wss.on("connection", socket => {
		sockets.push(socket);
		socket.on("message", data => {
			JSON.parse(data.toString());
		});
		socket.on("close", () => {
			const index = sockets.indexOf(socket);
			if (index !== -1) {
				sockets = sockets.splice(index, 1);
			}
		});
	});

	return {
		server: wss,
		broad(payload) {
			for (const socket of sockets) {
				socket.send(JSON.stringify(payload));
			}
		}
	};
}
