import type { Compiler, RspackOptionsNormalized } from "@rspack/core";
import type { Logger } from "./logger";
import type { Socket } from "net";
import type { FSWatcher, WatchOptions } from "chokidar";
import type { WebSocketServer, ClientConnection } from "./ws";
import type { RspackDevMiddleware } from "@rspack/dev-middleware";
import type {
	Application,
	RequestHandler as ExpressRequestHandler,
	ErrorRequestHandler as ExpressErrorRequestHandler
} from "express";
import type { Server } from "http";

import chokidar from "chokidar";
import http from "http";
import { createLogger } from "./logger";
import WebpackDevServer from "webpack-dev-server";
import express from "express";
import { rdm } from "@rspack/dev-middleware";
import { createWebsocketServer } from "./ws";

interface Middleware {
	name?: string;
	path?: string;
	middleware: ExpressErrorRequestHandler | ExpressRequestHandler;
}
interface Listener {
	name: string | Symbol;
	listener: (...args: any) => void;
}
type Host = "local-ip" | "local-ipv4" | "local-ipv6" | string;
type Port = number | string | "auto";

// copy from webpack-dev-server
export class RspackDevServer {
	options: RspackOptionsNormalized["devServer"];
	logger: Logger;
	staticWatchers: FSWatcher[];
	sockets: Socket[];
	app: Application;
	server: Server;
	private listeners: Listener[];
	private currentHash: string;
	private middleware: RspackDevMiddleware | undefined;
	// TODO: now only support 'ws'
	webSocketServer: WebSocketServer | undefined;

	constructor(public compiler: Compiler) {
		this.logger = createLogger("rspack-dev-server");
		this.staticWatchers = [];
		this.listeners = [];
		this.sockets = [];
		this.currentHash = "";
		this.options = compiler.options["devServer"];
	}

	static isAbsoluteURL(URL: string): boolean {
		return WebpackDevServer.isAbsoluteURL(URL);
	}

	static findIp(gateway: string): string | undefined {
		return WebpackDevServer.findIp(gateway);
	}

	static async internalIP(family: "v6" | "v4"): Promise<string | undefined> {
		return WebpackDevServer.internalIP(family);
	}

	static async internalIPSync(
		family: "v6" | "v4"
	): Promise<string | undefined> {
		return WebpackDevServer.internalIPSync(family);
	}

	static async getHostname(hostname: Host): Promise<string> {
		return WebpackDevServer.getHostname(hostname);
	}

	static async getFreePort(port: Port, host: string): Promise<string | number> {
		return WebpackDevServer.getFreePort(port, host);
	}

	static findCacheDir(): string {
		// TODO: we need remove the `webpack-dev-server` tag in WebpackDevServer;
		return "";
	}

	private getCompilerOptions(): RspackOptionsNormalized {
		return this.compiler.options;
	}

	sendMessage(
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

	watchFiles(watchPath: string | string[], watchOptions?: WatchOptions): void {
		const watcher = chokidar.watch(watchPath, watchOptions);

		// disabling refreshing on changing the content
		if (this.options.liveReload) {
			watcher.on("change", item => {
				if (this.webSocketServer) {
					this.sendMessage(
						this.webSocketServer.clients,
						"static-changed",
						item
					);
				}
			});
		}

		this.staticWatchers.push(watcher);
	}

	invalidate(callback = () => {}): void {
		if (this.middleware) {
			this.middleware.invalidate(callback);
		}
	}

	async start(): Promise<void> {
		this.initialize();
		this.createWebsocketServer();
		await new Promise(resolve =>
			this.server.listen(
				{
					port: this.options.port,
					host: "localhost"
				},
				() => {
					console.log(`begin server at http://localhost:${this.options.port}`);
					resolve({});
				}
			)
		);
	}

	startCallback(callback?: (err?: Error) => void): void {
		throw new Error("Method not implemented.");
	}
	stopCallback(callback?: (err?: Error) => void): void {
		throw new Error("Method not implemented.");
	}
	listen(port: Port, hostname: string, fn: (err?: Error) => void): void {
		throw new Error("Method not implemented.");
	}
	close(callback?: (err?: Error) => void): void {
		throw new Error("Method not implemented.");
	}

	async stop(): Promise<void> {
		await Promise.all(this.staticWatchers.map(watcher => watcher.close()));
		this.middleware = null;
		this.staticWatchers = [];
		if (this.server) {
			this.server.close();
		}
		if (this.webSocketServer) {
			await new Promise(resolve => {
				this.webSocketServer;
			});
		}
	}

	private initialize() {
		this.setupApp();
		this.setupDevMiddleware();
		this.setupWatchStaticFiles();
		this.setupMiddlewares();
		this.createServer();
	}

	private setupApp() {
		this.app = express();
	}

	private setupWatchStaticFiles() {
		if (this.options.static.watch === false) {
			return;
		}
		this.watchFiles(this.options.static.directory, this.options.static.watch);
	}

	private setupDevMiddleware() {
		this.middleware = rdm(this.compiler, this.options.devMiddleware);
	}

	private createWebsocketServer() {
		if (this.options.webSocketServer !== false) {
			this.webSocketServer = createWebsocketServer(this);
		}
	}

	private setupMiddlewares() {
		const middlewares: Middleware[] = [];
		middlewares.push({
			name: "rdm",
			middleware: this.middleware
		});

		middlewares.push({
			name: "express-static",
			middleware: express.static(this.options.static.directory)
		});

		middlewares.forEach(m => {
			if (m.path) {
				this.app.use(m.path, m.middleware);
			} else {
				this.app.use(m.middleware);
			}
		});
	}

	private createServer() {
		this.server = http.createServer(this.app);
	}
}
