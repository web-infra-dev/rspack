import { Compiler, MultiCompiler } from "@rspack/core";
import type { Socket } from "net";
import type { FSWatcher, WatchOptions } from "chokidar";
import rdm, { getRspackMemoryAssets } from "@rspack/dev-middleware";
import type { Server } from "http";
import fs from "fs";
import WebpackDevServer from "webpack-dev-server";
import type { ResolvedDevServer, DevServer } from "./config";

export class RspackDevServer extends WebpackDevServer {
	/**
	 * resolved after `normalizedOptions`
	 */
	options: ResolvedDevServer;
	staticWatchers: FSWatcher[];
	sockets: Socket[];
	server: Server;
	// @ts-expect-error
	public compiler: Compiler | MultiCompiler;
	webSocketServer: WebpackDevServer.WebSocketServerImplementation | undefined;

	constructor(options: DevServer, compiler: Compiler | MultiCompiler) {
		// @ts-expect-error
		super(options, compiler);
	}

	addAdditionEntires(compiler: Compiler) {
		const entries: string[] = [];

		// TODO: should use providerPlugin
		entries.push(this.getClientTransport());
		if (this.options.hot) {
			const hotUpdateEntryPath = require.resolve(
				"@rspack/dev-client/devServer"
			);
			entries.push(hotUpdateEntryPath);

			if (compiler.options.builtins.react?.refresh) {
				const reactRefreshEntryPath = require.resolve(
					"@rspack/dev-client/react-refresh"
				);
				entries.push(reactRefreshEntryPath);
			}
		}

		const devClientEntryPath = require.resolve("@rspack/dev-client");
		entries.push(devClientEntryPath);
		for (const key in compiler.options.entry) {
			compiler.options.entry[key].import.unshift(...entries);
		}
	}

	watchFiles(watchPath: string | string[], watchOptions?: WatchOptions): void {
		const chokidar = require("chokidar");
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
			const compilers =
				this.compiler instanceof MultiCompiler
					? this.compiler.compilers
					: [this.compiler];
			compilers.forEach(compiler => {
				const compilers =
					compiler instanceof MultiCompiler ? compiler.compilers : [compiler];
				compilers.forEach(compiler => {
					compiler.options.devServer ??= {};
					compiler.options.builtins.react ??= {};
					if (this.options.hot) {
						compiler.options.builtins.react.refresh ??= true;
						compiler.options.builtins.react.development ??= true;
					} else if (compiler.options.builtins.react.refresh) {
						this.logger.warn(
							"builtins.react.refresh needs builtins.react.development and devServer.hot enabled"
						);
					}
				});

				this.addAdditionEntires(compiler);
			});
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

		if (this.options.setupExitSignals) {
			const signals = ["SIGINT", "SIGTERM"];

			let needForceShutdown = false;

			signals.forEach(signal => {
				const listener = () => {
					if (needForceShutdown) {
						process.exit();
					}

					this.logger.info(
						"Gracefully shutting down. To force exit, press ^C again. Please wait..."
					);

					needForceShutdown = true;

					this.stopCallback(() => {
						if (typeof this.compiler.close === "function") {
							this.compiler.close(() => {
								process.exit();
							});
						} else {
							process.exit();
						}
					});
				};

				// @ts-expect-error: `listeners` is private function in base class.
				this.listeners.push({ name: signal, listener });

				process.on(signal, listener);
			});
		}

		// Proxy WebSocket without the initial http request
		// https://github.com/chimurai/http-proxy-middleware#external-websocket-upgrade
		// @ts-expect-error: `webSocketProxies` is private function in base class.
		this.webSocketProxies.forEach(webSocketProxy => {
			this.server.on("upgrade", webSocketProxy.upgrade);
		}, this);
	}

	private setupDevMiddleware() {
		// @ts-ignored
		this.middleware = rdm(this.compiler, this.options.devMiddleware);
	}

	private setupMiddlewares() {
		const middlewares: WebpackDevServer.Middleware[] = [];
		const compilers =
			this.compiler instanceof MultiCompiler
				? this.compiler.compilers
				: [this.compiler];

		if (Array.isArray(this.options.static)) {
			this.options.static.forEach(staticOptions => {
				staticOptions.publicPath.forEach(publicPath => {
					compilers.forEach(compiler => {
						if (compiler.options.builtins.noEmitAssets) {
							middlewares.push({
								name: "rspack-memory-assets",
								path: publicPath,
								middleware: getRspackMemoryAssets(compiler, this.middleware)
							});
						}
					});
				});
			});
		}

		compilers.forEach(compiler => {
			if (compiler.options.experiments.lazyCompilation) {
				middlewares.push({
					middleware: (req, res, next) => {
						if (req.url.indexOf("/lazy-compilation-web/") > -1) {
							const path = req.url.replace("/lazy-compilation-web/", "");
							if (fs.existsSync(path)) {
								compiler.rebuild(new Set([path]), new Set(), error => {
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
		});

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
