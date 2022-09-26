import type { RspackOptions, ResolvedRspackOptions } from "@rspack/core";
import type { DevMiddleware } from "@rspack/dev-middleware";
import { rdm } from "@rspack/dev-middleware";

import type {
	Express,
	RequestHandler as ExpressRequestHandler,
	ErrorRequestHandler as ExpressErrorRequestHandler
} from "express";
import type { FSWatcher, WatchOptions } from "chokidar";
import type { Server as HttpServer } from "http";
import http from "http";
import express from "express";
import chokidar from "chokidar";
import { Rspack } from "@rspack/core";
import { createWebSocketServer } from "./ws";
import type { ClientConnection, WebSocketServer } from "./ws";

interface Middleware {
	name?: string;
	path?: string;
	middleware: ExpressErrorRequestHandler | ExpressRequestHandler;
}

interface Server {
	start(): Promise<void>;
	stop(): Promise<void>;
}

export async function createServer(
	rspackOptions: RspackOptions
): Promise<Server> {
	const compiler = new Rspack(rspackOptions);

	const options: ResolvedRspackOptions["dev"] = compiler.options.dev;
	let staticWatchers: FSWatcher[] = [];
	let server: HttpServer | undefined;
	let app: Express | undefined;
	let middleware: DevMiddleware | undefined;
	let webSocketServer: WebSocketServer | undefined;

	function sendMessage(
		clients: ClientConnection[],
		type: string,
		data?: any,
		params?: any
	) {
		for (const client of clients) {
			if (client.readyState === 1) {
				client.send(JSON.stringify({ type, data, params }));
			}
		}
	}

	function watchFiles(
		watchpath: string | string[],
		watchOptions: WatchOptions
	) {
		console.log("watchpath", watchpath);
		console.trace();
		const watcher = chokidar.watch(watchpath, watchOptions);
		if (options.liveReload) {
			watcher.on("change", item => {
				if (!webSocketServer) {
					return;
				}
				sendMessage(webSocketServer.clients, "static-changed", item);
			});
		}
		staticWatchers.push(watcher);
	}

	function createServer() {
		server = http.createServer(app);
	}

	function initialize() {
		setupApp();
		setupDevMiddleware();
		setupWatchStaticFiles();
		setupMiddlewares();
		createServer();
	}

	function setupApp() {
		app = express();
	}

	function setupWatchStaticFiles() {
		if (options.static.watch === false) {
			return;
		}
		watchFiles(options.static.directory, options.static.watch);
	}

	function setupDevMiddleware() {
		middleware = rdm(compiler, options.devMiddleware);
	}

	function setupMiddlewares() {
		const middlewares: Middleware[] = [];
		middlewares.push({
			name: "rdm",
			middleware: middleware
		});

		middlewares.push({
			name: "express-static",
			middleware: express.static(options.static.directory)
		});

		middlewares.forEach(m => {
			if (m.path) {
				app.use(m.path, m.middleware);
			} else {
				app.use(m.middleware);
			}
		});
	}

	function setupWebSocketServer() {
		webSocketServer = createWebSocketServer(server, {});
	}

	return {
		async start() {
			initialize();
			setupWebSocketServer();

			await new Promise(resolve =>
				server.listen(
					{
						port: options.port,
						host: "localhost"
					},
					() => {
						console.log(`begin server at http://localhost:${options.port}`);
						resolve({});
					}
				)
			);
		},
		async stop() {
			await Promise.all(staticWatchers.map(watcher => watcher.close()));
			middleware = null;
			staticWatchers = [];
			if (server) {
				server.close();
			}
			await webSocketServer.stop();
		}
	};
}
