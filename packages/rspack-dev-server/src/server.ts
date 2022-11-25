import type { Compiler, Dev, RspackOptionsNormalized } from "@rspack/core";
import type { Logger } from "./logger";
import type { Socket } from "net";
import type { FSWatcher, WatchOptions } from "chokidar";
import type { WebSocketServer, ClientConnection } from "./ws";
import type {
	Application,
	RequestHandler as ExpressRequestHandler,
	ErrorRequestHandler as ExpressErrorRequestHandler
} from "express";
import type { DevMiddleware } from "@rspack/dev-middleware";
import type { Server } from "http";
import type { ResolvedDev } from "./config";

import chokidar from "chokidar";
import http from "http";
import { createLogger } from "./logger";
import WebpackDevServer from "webpack-dev-server";
import express from "express";
import rdm from "@rspack/dev-middleware";
import { createWebsocketServer } from "./ws";
import { resolveDevOptions } from "./config";

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
	options: ResolvedDev;
	logger: Logger;
	staticWatchers: FSWatcher[];
	sockets: Socket[];
	app: Application;
	server: Server;
	private listeners: Listener[];
	private currentHash: string;
	private middleware: DevMiddleware | undefined;
	// TODO: now only support 'ws'
	webSocketServer: WebSocketServer | undefined;

	constructor(public compiler: Compiler) {
		this.logger = createLogger("rspack-dev-server");
		this.staticWatchers = [];
		this.listeners = [];
		this.sockets = [];
		this.currentHash = "";
		this.options = this.normalizeOptions(compiler.options.devServer);
		this.rewriteCompilerOptions();
		this.addAdditionEntires();
	}

	normalizeOptions(dev: Dev = {}) {
		return resolveDevOptions(dev, this.compiler.options);
	}

	rewriteCompilerOptions() {
		this.compiler.options.devServer = this.options;
		if (!this.compiler.options.builtins.react) {
			this.compiler.options.builtins.react = {};
		}
		if (this.compiler.options.builtins.react?.development !== true) {
			this.logger.warn('The value of react.development is not being as expected and has been automatically changed to true.')
		}
		this.compiler.options.builtins.react.development = true;
	}

	addAdditionEntires() {
		const entries: string[] = [];

		if (this.options.hot) {
			const hotUpdateEntryPath = require.resolve(
				"@rspack/dev-client/devServer"
			);
			entries.push(hotUpdateEntryPath);

			const cssHotEntryPath = require.resolve("@rspack/dev-client/css");

			entries.push(cssHotEntryPath);

			const reactRefreshEntryPath = require.resolve(
				"@rspack/dev-client/react-refresh"
			);
			entries.push(reactRefreshEntryPath);
		}

		const devClientEntryPath = require.resolve("@rspack/dev-client");
		entries.push(devClientEntryPath);
		for (const key in this.compiler.options.entry) {
			this.compiler.options.entry[key].unshift(...entries);
		}
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

	static async getHostname(hostname?: Host): Promise<string> {
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
			// TODO: remove this after we had memory filesystem
			if (this.options.hot) {
				return;
			}

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

	invalidate(callback = () => { }): void {
		if (this.middleware) {
			this.middleware.invalidate(callback);
		}
	}

	async start(): Promise<void> {
		this.setupHooks();
		this.setupApp();
		this.createServer();
		this.setupWatchStaticFiles();
		this.createWebsocketServer();
		this.setupDevMiddleware();
		this.setupMiddlewares();
		const host = await RspackDevServer.getHostname(this.options.host);
		const port = await RspackDevServer.getFreePort(this.options.port, host);
		await new Promise(resolve =>
			this.server.listen(
				{
					port,
					host
				},
				() => {
					this.logger.info(`Loopback: http://localhost:${port}`);
					let internalIPv4 = WebpackDevServer.internalIPSync("v4");
					this.logger.info(
						`Your Network (IPV4) http://${internalIPv4}:${port}`
					);
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
				this.webSocketServer.implementation.close(() => {
					resolve(void 0);
				});
				for (const client of this.webSocketServer.clients) client.terminate();
			});
		}
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
		// @ts-ignored
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

		// Todo Add options
		const connectHistoryApiFallback = require("connect-history-api-fallback");
		middlewares.push({
			name: "[connect-history-api-fallback]",
			middleware: connectHistoryApiFallback({
				verbose: true,
				logger: console.log.bind(console)
			})
		});
		middlewares.push({
			name: "express-static",
			path: this.compiler.options.output.publicPath ?? "/",
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

	private setupHooks() {
		this.compiler.hooks.done.tap("dev-server", stats => {
			// send Message
			if (this.webSocketServer) {
				this.sendMessage(this.webSocketServer.clients, "ok"); // TODO: send hash
			}
		});
	}
}
