/**
 * The following code is modified based on
 * https://github.com/webpack/webpack-dev-server/blob/b0f15ace0123c125d5870609ef4691c141a6d187/lib/Server.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack-dev-server/blob/b0f15ace0123c125d5870609ef4691c141a6d187/LICENSE
 */
import { Compiler, MultiCompiler } from "@rspack/core";
import type { Socket } from "net";
import type { FSWatcher } from "chokidar";
import { getRspackMemoryAssets, rdm } from "@rspack/dev-middleware";
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
		super(
			{
				...options,
				setupMiddlewares: (middlewares, devServer) => {
					const webpackDevMiddlewareIndex = middlewares.findIndex(
						mid => mid.name === "webpack-dev-middleware"
					);
					const compilers =
						compiler instanceof MultiCompiler ? compiler.compilers : [compiler];
					if (compilers[0].options.builtins.noEmitAssets) {
						if (Array.isArray(this.options.static)) {
							const memoryAssetsMiddlewares = this.options.static.flatMap(
								staticOptions => {
									return staticOptions.publicPath.flatMap(publicPath => {
										return compilers.map(compiler => {
											return {
												name: "rspack-memory-assets",
												path: publicPath,
												middleware: getRspackMemoryAssets(
													compiler,
													this.middleware
												)
											};
										});
									});
								}
							);
							middlewares.splice(
								webpackDevMiddlewareIndex,
								0,
								...memoryAssetsMiddlewares
							);
						}
					}

					options.setupMiddlewares?.call(this, middlewares, devServer);
					return middlewares;
				}
			},
			compiler as any
		);
	}

	addAdditionalEntries(compiler: Compiler) {
		const additionalEntries: string[] = [];
		// @ts-expect-error
		const isWebTarget = WebpackDevServer.isWebTarget(compiler);
		// inject runtime first, avoid other additional entry after transfrom depend on it
		if (this.options.hot) {
			if (compiler.options.builtins.react?.refresh) {
				const reactRefreshEntryPath = require.resolve(
					"@rspack/dev-client/react-refresh"
				);
				additionalEntries.push(reactRefreshEntryPath);
			}
			const hotUpdateEntryPath = require.resolve("webpack/hot/dev-server");
			additionalEntries.push(hotUpdateEntryPath);
		}
		if (this.options.client && isWebTarget) {
			let webSocketURLStr = "";

			if (this.options.webSocketServer) {
				const webSocketURL = this.options.client
					.webSocketURL as WebpackDevServer.WebSocketURL;
				const webSocketServer = this.options.webSocketServer;
				const searchParams = new URLSearchParams();

				let protocol: string;

				// We are proxying dev server and need to specify custom `hostname`
				if (typeof webSocketURL.protocol !== "undefined") {
					protocol = webSocketURL.protocol;
				} else {
					protocol = this.options.server.type === "http" ? "ws:" : "wss:";
				}

				searchParams.set("protocol", protocol);

				if (typeof webSocketURL.username !== "undefined") {
					searchParams.set("username", webSocketURL.username);
				}

				if (typeof webSocketURL.password !== "undefined") {
					searchParams.set("password", webSocketURL.password);
				}

				let hostname: string;

				// SockJS is not supported server mode, so `hostname` and `port` can't specified, let's ignore them
				// TODO show warning about this
				const isSockJSType = webSocketServer.type === "sockjs";

				// We are proxying dev server and need to specify custom `hostname`
				if (typeof webSocketURL.hostname !== "undefined") {
					hostname = webSocketURL.hostname;
				}
				// Web socket server works on custom `hostname`, only for `ws` because `sock-js` is not support custom `hostname`
				else if (
					typeof webSocketServer.options.host !== "undefined" &&
					!isSockJSType
				) {
					hostname = webSocketServer.options.host;
				}
				// The `host` option is specified
				else if (typeof this.options.host !== "undefined") {
					hostname = this.options.host;
				}
				// The `port` option is not specified
				else {
					hostname = "0.0.0.0";
				}

				searchParams.set("hostname", hostname);

				let port: number | string;

				// We are proxying dev server and need to specify custom `port`
				if (typeof webSocketURL.port !== "undefined") {
					port = webSocketURL.port;
				}
				// Web socket server works on custom `port`, only for `ws` because `sock-js` is not support custom `port`
				else if (
					typeof webSocketServer.options.port !== "undefined" &&
					!isSockJSType
				) {
					port = webSocketServer.options.port;
				}
				// The `port` option is specified
				else if (typeof this.options.port === "number") {
					port = this.options.port;
				}
				// The `port` option is specified using `string`
				else if (
					typeof this.options.port === "string" &&
					this.options.port !== "auto"
				) {
					port = Number(this.options.port);
				}
				// The `port` option is not specified or set to `auto`
				else {
					port = "0";
				}

				searchParams.set("port", String(port));

				let pathname = "";

				// We are proxying dev server and need to specify custom `pathname`
				if (typeof webSocketURL.pathname !== "undefined") {
					pathname = webSocketURL.pathname;
				}
				// Web socket server works on custom `path`
				else if (
					typeof webSocketServer.options.prefix !== "undefined" ||
					typeof webSocketServer.options.path !== "undefined"
				) {
					pathname =
						webSocketServer.options.prefix || webSocketServer.options.path;
				}

				searchParams.set("pathname", pathname);

				const client = /** @type {ClientConfiguration} */ this.options.client;

				if (typeof client.logging !== "undefined") {
					searchParams.set("logging", client.logging);
				}

				if (typeof client.progress !== "undefined") {
					searchParams.set("progress", String(client.progress));
				}

				if (typeof client.overlay !== "undefined") {
					searchParams.set(
						"overlay",
						typeof client.overlay === "boolean"
							? String(client.overlay)
							: JSON.stringify(client.overlay)
					);
				}

				if (typeof client.reconnect !== "undefined") {
					searchParams.set(
						"reconnect",
						typeof client.reconnect === "number"
							? String(client.reconnect)
							: "10"
					);
				}

				if (typeof this.options.hot !== "undefined") {
					searchParams.set("hot", String(this.options.hot));
				}

				if (typeof this.options.liveReload !== "undefined") {
					searchParams.set("live-reload", String(this.options.liveReload));
				}

				webSocketURLStr = searchParams.toString();
			}

			additionalEntries.push(
				`${require.resolve(
					"webpack-dev-server/client/index.js"
				)}?${webSocketURLStr}`
			);
		}

		for (const key in compiler.options.entry) {
			compiler.options.entry[key].import.unshift(...additionalEntries);
		}
	}

	getClientTransport(): string {
		// WARNING: we can't use `super.getClientTransport`,
		// because we doesn't had same directory structure.
		let clientImplementation: string | undefined;
		let clientImplementationFound = true;
		const isKnownWebSocketServerImplementation =
			this.options.webSocketServer &&
			typeof this.options.webSocketServer.type === "string" &&
			(this.options.webSocketServer.type === "ws" ||
				this.options.webSocketServer.type === "sockjs");

		let clientTransport: string | undefined;

		if (this.options.client) {
			if (typeof this.options.client.webSocketTransport !== "undefined") {
				clientTransport = this.options.client.webSocketTransport;
			} else if (isKnownWebSocketServerImplementation) {
				// @ts-expect-error: TS cannot infer webSocketServer is narrowed
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
						"webpack-dev-server/client/clients/SockJSClient"
					);
				} else if (clientTransport === "ws") {
					clientImplementation = require.resolve(
						"webpack-dev-server/client/clients/WebSocketClient"
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
		const compilers =
			this.compiler instanceof MultiCompiler
				? this.compiler.compilers
				: [this.compiler];

		compilers.forEach(compiler => {
			const mode = compiler.options.mode || process.env.NODE_ENV;
			if (this.options.hot) {
				if (mode === "production") {
					this.logger.warn(
						"Hot Module Replacement (HMR) is enabled for the production build. \n" +
							"Make sure to disable HMR for production by setting `devServer.hot` to `false` in the configuration."
					);
				}
				compiler.options.devServer ??= {};
				compiler.options.devServer.hot = true;
				compiler.options.builtins.react ??= {};
				compiler.options.builtins.react.refresh ??= true;
				compiler.options.builtins.react.development ??= true;
			} else if (compiler.options.builtins.react.refresh) {
				if (mode === "production") {
					this.logger.warn(
						"React Refresh runtime should not be included in the production bundle.\n" +
							"Make sure to disable React Refresh for production by setting `builtins.react.refresh` to `false` in the configuration."
					);
				} else {
					this.logger.warn(
						"The `builtins.react.refresh` needs `builtins.react.development` and `devServer.hot` enabled"
					);
				}
			}
		});

		if (this.options.webSocketServer) {
			compilers.forEach(compiler => {
				this.addAdditionalEntries(compiler);

				compiler.options.builtins.provide = {
					...compiler.options.builtins.provide,
					__webpack_dev_server_client__: [this.getClientTransport()]
				};
			});
		}

		// @ts-expect-error: `setupHooks` is private function in base class.
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
		// @ts-expect-error
		this.middleware = rdm(this.compiler, this.options.devMiddleware);
	}

	private setupMiddlewares() {
		const middlewares: WebpackDevServer.Middleware[] = [];
		const compilers =
			this.compiler instanceof MultiCompiler
				? this.compiler.compilers
				: [this.compiler];

		// if (Array.isArray(this.options.static)) {
		// 	this.options.static.forEach(staticOptions => {
		// 		staticOptions.publicPath.forEach(publicPath => {
		// 			compilers.forEach(compiler => {
		// 				if (compiler.options.builtins.noEmitAssets) {
		// 					middlewares.push({
		// 						name: "rspack-memory-assets",
		// 						path: publicPath,
		// 						middleware: getRspackMemoryAssets(compiler, this.middleware)
		// 					});
		// 				}
		// 			});
		// 		});
		// 	});
		// }

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
}
