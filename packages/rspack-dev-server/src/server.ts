import type { Compiler, Dev } from "@rspack/core";
import type { Socket } from "net";
import type { FSWatcher, WatchOptions } from "chokidar";
import rdm, { getRspackMemoryAssets } from "@rspack/dev-middleware";
import type { Server } from "http";
import type { ResolvedDev } from "./config";
import fs from "fs";
import chokidar from "chokidar";
import WebpackDevServer from "webpack-dev-server";

export class RspackDevServer extends WebpackDevServer {
	/**
	 * resolved after `normalizedOptions`
	 */
	options: ResolvedDev;
	staticWatchers: FSWatcher[];
	sockets: Socket[];
	server: Server;
	// @ts-expect-error
	public compiler: Compiler;
	webSocketServer: WebpackDevServer.WebSocketServerImplementation | undefined;

	constructor(compiler: Compiler) {
		compiler.options.devServer = compiler.options.devServer ?? {};
		// @ts-expect-error
		super(compiler.options.devServer, compiler);
		this.staticWatchers = [];
		this.sockets = [];
	}

	addAdditionEntires() {
		const entries: string[] = [];

		// TODO: should use providerPlugin
		entries.push(this.getClientTransport());
		if (this.options.hot) {
			const hotUpdateEntryPath = require.resolve(
				"@rspack/dev-client/devServer"
			);
			entries.push(hotUpdateEntryPath);

			if (this.compiler.options.builtins.react?.refresh) {
				const reactRefreshEntryPath = require.resolve(
					"@rspack/dev-client/react-refresh"
				);
				entries.push(reactRefreshEntryPath);
			}
		}

		const devClientEntryPath = require.resolve("@rspack/dev-client");
		entries.push(devClientEntryPath);
		for (const key in this.compiler.options.entry) {
			this.compiler.options.entry[key].import.unshift(...entries);
		}
	}

	static findCacheDir(): string {
		// TODO: we need remove the `webpack-dev-server` tag in WebpackDevServer;
		return "";
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

	getClientTransport(): string {
		// WARNING: we can't use `super.getClientTransport`,
		// because we doesn't had same directory structure.
		// and TODO: we need impelement `webpack.providerPlugin`
		let clientImplementation: string | undefined;
		let clientImplementationFound = true;
		const isKnownWebSocketServerImplementation =
			this.options.webSocketServer &&
			typeof this.options.webSocketServer.type === "string" &&
			(this.options.webSocketServer.type === "ws" ||
				this.options.webSocketServer.type === "sockjs");

		let clientTransport: string | undefined;

		if (this.options.client) {
			if (
				// @ts-ignore
				typeof this.options.client.webSocketTransport !== "undefined"
			) {
				// @ts-ignore
				clientTransport = this.options.client.webSocketTransport;
			} else if (isKnownWebSocketServerImplementation) {
				// @ts-ignore
				clientTransport = this.options.webSocketServer.type;
			} else {
				clientTransport = "ws";
			}
		} else {
			clientTransport = "ws";
		}

		switch (typeof clientTransport) {
			case "string":
				// could be 'sockjs', 'ws', or a path that should be required
				if (clientTransport === "sockjs") {
					clientImplementation = require.resolve(
						"@rspack/dev-client/clients/SockJSClient"
					);
				} else if (clientTransport === "ws") {
					clientImplementation = require.resolve(
						"@rspack/dev-client/clients/WebSocketClient"
					);
				} else {
					try {
						clientImplementation = require.resolve(clientTransport);
						throw Error("Do not support custom ws client now");
					} catch (e) {
						clientImplementationFound = false;
					}
				}
				break;
			default:
				clientImplementationFound = false;
		}
		if (!clientImplementationFound) {
			throw new Error(
				`${
					!isKnownWebSocketServerImplementation
						? "When you use custom web socket implementation you must explicitly specify client.webSocketTransport. "
						: ""
				}client.webSocketTransport must be a string denoting a default implementation (e.g. 'sockjs', 'ws') or a full path to a JS file via require.resolve(...) which exports a class `
			);
		}

		return clientImplementation;
	}

	async initialize() {
		if (this.options.webSocketServer) {
			this.addAdditionEntires();
		}

		this.setupHooks();
		// @ts-expect-error: `setupApp` is private function in base class.
		this.setupApp();
		// @ts-expect-error: `setupHostHeaderCheck` is private function in base class.
		this.setupHostHeaderCheck();
		this.setupDevMiddleware();
		// @ts-expect-error: `setupBuiltInRoutes` is private function in base class.
		this.setupBuiltInRoutes();
		// @ts-expect-error: `setupWatchFiles` is private function in base class.
		this.setupWatchFiles();
		// @ts-expect-error: `setupWatchStaticFiles` is private function in base class.
		this.setupWatchStaticFiles();
		this.setupMiddlewares();
		// @ts-expect-error: `createServer` is private function in base class.
		this.createServer();
	}

	async start(): Promise<void> {
		// @ts-expect-error: `normalizeOptions` is private function in base class.
		await this.normalizeOptions();

		if (!this.compiler.options.builtins.react) {
			this.compiler.options.builtins.react = {};
		}
		this.compiler.options.builtins.react.development =
			this.compiler.options.builtins.react.development ?? true;
		if (this.options.hot) {
			this.compiler.options.builtins.react.refresh =
				this.compiler.options.builtins.react.refresh ?? true;
		} else if (this.compiler.options.builtins.react.refresh) {
			this.logger.warn(
				"[Builtins] react.refresh need react.development and devServer.hot enabled."
			);
		}

		if (this.options.ipc) {
			await new Promise((resolve, reject) => {
				const net = require("net");
				const socket = new net.Socket();

				socket.on("error", error => {
					if (error.code === "ECONNREFUSED") {
						fs.unlinkSync(this.options.ipc);
						resolve(undefined);
						return;
					} else if (error.code === "ENOENT") {
						resolve(undefined);
						return;
					}
					reject(error);
				});

				socket.connect({ path: this.options.ipc }, () => {
					throw new Error(`IPC "${this.options.ipc}" is already used`);
				});
			});
		} else {
			this.options.host = await RspackDevServer.getHostname(this.options.host);
			this.options.port = await RspackDevServer.getFreePort(
				this.options.port,
				this.options.host
			);
		}

		await this.initialize();

		const listenOptions = this.options.ipc
			? { path: this.options.ipc }
			: { host: this.options.host, port: this.options.port };

		await new Promise(resolve => {
			this.server.listen(listenOptions, () => {
				resolve(undefined);
			});
		});

		if (this.options.ipc) {
			// chmod 666 (rw rw rw)
			const READ_WRITE = 438;

			await fs.promises.chmod(this.options.ipc, READ_WRITE);
		}

		if (this.options.webSocketServer) {
			// @ts-expect-error: private function
			this.createWebSocketServer();
		}

		// @ts-expect-error: private function
		this.logStatus();
	}

	private setupDevMiddleware() {
		// @ts-ignored
		this.middleware = rdm(this.compiler, this.options.devMiddleware);
	}

	private setupMiddlewares() {
		const middlewares: WebpackDevServer.Middleware[] = [];

		if (Array.isArray(this.options.static)) {
			this.options.static.forEach(staticOptions => {
				staticOptions.publicPath.forEach(publicPath => {
					middlewares.push({
						name: "rspack-memory-assets",
						path: publicPath,
						middleware: getRspackMemoryAssets(this.compiler)
					});
				});
			});
		}

		if (this.compiler.options.experiments.lazyCompilation) {
			middlewares.push({
				middleware: (req, res, next) => {
					if (req.url.indexOf("/lazy-compilation-web/") > -1) {
						const path = req.url.replace("/lazy-compilation-web/", "");
						if (fs.existsSync(path)) {
							this.compiler.rebuild(new Set([path]), new Set(), error => {
								if (error) {
									throw error;
								}
								res.write("");
								res.end();
								console.log("lazy compiler success");
							});
						}
					}
				}
			});
		}

		middlewares.forEach(middleware => {
			if (typeof middleware === "function") {
				this.app.use(middleware);
			} else if (typeof middleware.path !== "undefined") {
				this.app.use(middleware.path, middleware.middleware);
			} else {
				this.app.use(middleware.middleware);
			}
		});

		// @ts-expect-error
		super.setupMiddlewares();
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
