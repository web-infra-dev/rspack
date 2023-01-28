import type { Compiler, Dev } from "@rspack/core";
import type { Socket } from "net";
import type { FSWatcher, WatchOptions } from "chokidar";
import rdm, { getRspackMemoryAssets } from "@rspack/dev-middleware";
import type { Server } from "http";
import type { ResolvedDev } from "./config";
import fs from "fs";
import chokidar from "chokidar";
import http from "http";
import WebpackDevServer from "webpack-dev-server";
import express from "express";
import { resolveDevOptions } from "./config";

export class RspackDevServer extends WebpackDevServer {
	options: ResolvedDev;
	staticWatchers: FSWatcher[];
	sockets: Socket[];
	server: Server;
	private middleware: ReturnType<typeof rdm>;
	// @ts-expect-error
	public compiler: Compiler;
	webSocketServer: WebpackDevServer.WebSocketServerImplementation | undefined;

	constructor(compiler: Compiler) {
		// @ts-expect-error
		super({}, compiler);
		this.staticWatchers = [];
		this.sockets = [];
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

	async start(): Promise<void> {
		this.setupHooks();
		this.setupApp();
		this.createServer();
		this.setupWatchStaticFiles();
		if (this.options.webSocketServer) {
			// @ts-expect-error: it a private function defined in `WebpackDevServer`.
			this.createWebSocketServer();
		}
		this.setupDevMiddleware();
		this.setupMiddlewares();

		const host = await RspackDevServer.getHostname(this.options.host);
		const port = await RspackDevServer.getFreePort(this.options.port, host);
		this.options.port = port;
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
					if (this.options.historyApiFallback) {
						this.logger.info(
							`404s will fallback to '${
								this.options.historyApiFallback.index || "/index.html"
							}'`
						);
					}
					resolve({});
				}
			)
		);
	}

	async stop(): Promise<void> {
		this.compiler.close(() => {});
		await Promise.all(this.staticWatchers.map(watcher => watcher.close()));
		this.staticWatchers = [];

		if (this.middleware) {
			await new Promise((resolve, reject) => {
				this.middleware.close((error: Error) => {
					if (error) {
						reject(error);
						return;
					}
					resolve(undefined);
				});
			});
		}
		this.middleware = null;

		if (this.server) {
			this.server.close();
		}
		if (this.webSocketServer) {
			await new Promise(resolve => {
				this.webSocketServer.implementation.close(() => {
					this.webSocketServer = null;
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

	private setupMiddlewares() {
		const options = this.options;
		const middlewares: WebpackDevServer.Middleware[] = [];
		middlewares.push({
			name: "rdm",
			middleware: this.middleware
		});

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

		if (this.options.historyApiFallback) {
			const connectHistoryApiFallback = require("connect-history-api-fallback");
			const { historyApiFallback } = this.options;

			if (
				typeof historyApiFallback.logger === "undefined" &&
				!historyApiFallback.verbose
			) {
				(historyApiFallback as any).logger = this.logger.log.bind(
					this.logger,
					"[connect-history-api-fallback]"
				);
			}

			middlewares.push({
				name: "connect-history-api-fallback",
				middleware: connectHistoryApiFallback(historyApiFallback)
			});
		}

		/**
		 * supports three kinds of proxy configuration
		 * {context: 'xxxx', target: 'yyy}
		 * {['xxx']: { target: 'yyy}}
		 * [{context: 'xxx',target:'yyy'}, {context: 'aaa', target: 'zzzz'}]
		 */
		if (typeof options.proxy !== "undefined") {
			const { createProxyMiddleware } = require("http-proxy-middleware");
			function getProxyMiddleware(proxyConfig) {
				if (proxyConfig.target) {
					const context = proxyConfig.context || proxyConfig.path;
					return createProxyMiddleware(context, proxyConfig);
				}
				if (proxyConfig.router) {
					return createProxyMiddleware(proxyConfig);
				}
			}
			if (!Array.isArray(options.proxy)) {
				if (
					Object.prototype.hasOwnProperty.call(options.proxy, "target") ||
					Object.prototype.hasOwnProperty.call(options.proxy, "router")
				) {
					options.proxy = [options.proxy];
				} else {
					options.proxy = Object.keys(options.proxy).map(context => {
						let proxyOptions;
						// For backwards compatibility reasons.
						const correctedContext = context
							.replace(/^\*$/, "**")
							.replace(/\/\*$/, "");

						if (
							typeof (/** @type {ProxyConfigMap} */ options.proxy[context]) ===
							"string"
						) {
							proxyOptions = {
								context: correctedContext,
								target:
									/** @type {ProxyConfigMap} */
									options.proxy[context]
							};
						} else {
							proxyOptions = {
								// @ts-ignore
								.../** @type {ProxyConfigMap} */ options.proxy[context]
							};
							proxyOptions.context = correctedContext;
						}

						return proxyOptions;
					});
				}
			}
			options.proxy.forEach(proxyConfigOrCallback => {
				const proxyConfig =
					typeof proxyConfigOrCallback === "function"
						? proxyConfigOrCallback()
						: proxyConfigOrCallback;

				const handler = async (req, res, next) => {
					let proxyMiddleware = getProxyMiddleware(proxyConfig);
					const isByPassFuncDefined = typeof proxyConfig.bypass === "function";
					const bypassUrl = isByPassFuncDefined
						? await proxyConfig.bypass(req, res, proxyConfig)
						: null;
					if (typeof bypassUrl === "boolean") {
						req.url = null;
						next();
					} else if (typeof bypassUrl === "string") {
						req.url = bypassUrl;
					} else if (proxyMiddleware) {
						return proxyMiddleware(req, res, next);
					} else {
						next();
					}
				};
				middlewares.push({
					name: "http-proxy-middleware",
					middleware: handler
				});
				middlewares.push({
					name: "http-proxy-middleware-error-handler",
					middleware: (error, req, res, next) => handler(req, res, next)
				});
			});
		}
		const publicPath =
			this.compiler.options.output.publicPath === "auto"
				? ""
				: this.compiler.options.output.publicPath;
		middlewares.push({
			name: "rspack-memory-assets",
			path: publicPath,
			middleware: getRspackMemoryAssets(this.compiler)
		});
		middlewares.push({
			name: "express-static",
			path: publicPath,
			middleware: express.static(this.options.static.directory)
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
