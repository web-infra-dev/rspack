import type * as http from "http";
import type { ResolvedRspackOptions, RspackOptions } from "../config";
import type { FSWatcher } from "chokidar";
import type { MiddlewareServer } from "./middleware";
import { createWebSocketServer, WebSocketServer } from "./ws";
import { closeHttpServer, createHttpServer } from "./http";
import { openBrowser } from "./open";
import { createWatcher } from "./watch";
import { createMiddleware } from "./middleware";
import { resolveOptions } from "../config";

export interface Server {
	http: http.Server;
	ws: WebSocketServer;
	watcher: FSWatcher;
	connect: MiddlewareServer;
	options: ResolvedRspackOptions;
	start(): Promise<void>;
	close(): Promise<void>;
}

function createServer(userOptions: RspackOptions): Server {
	// prevent modify,
	// maybe we can use `cloneDeep`, or `resolveOptions(resolvedOptions)`
	const options = resolveOptions(userOptions);
	const app = createMiddleware(options);
	const http = createHttpServer(app);
	const ws = createWebSocketServer();
	const watcher = createWatcher(options);

	http.on("upgrade", (req, socket, head) => {
		if (req.headers["sec-websocket-protocol"] !== "web-server") {
			return;
		}
		ws.server.handleUpgrade(req, socket, head, client => {
			ws.server.emit("connection", client, req);
		});
	});

	const server = {
		watcher,
		connect: app,
		http,
		ws,
		options,
		async start() {
			const protocol = "http";
			const hostname = "localhost";
			const port = options.dev.port;

			http.listen(port, hostname, () => {
				console.log("start http server");
			});

			if (options.dev.port) {
				const url = `${protocol}://${hostname}:${port}`;
				await openBrowser(url);
			}
		},
		async close() {
			watcher.close();
			ws.server.close();
			closeHttpServer(http);
		}
	};

	return server;
}

export { createServer };
