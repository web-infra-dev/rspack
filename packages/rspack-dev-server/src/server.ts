// @ts-nocheck
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
import rdm from "webpack-dev-middleware";
import type { Server } from "http";
import fs from "fs";
import WebpackDevServer from "webpack-dev-server";
import type { ResolvedDevServer, DevServer } from "./config";
import { getRspackMemoryAssets } from "./middleware";
import { applyDevServerPatch } from "./patch";

applyDevServerPatch();

export class RspackDevServer extends WebpackDevServer {
	/**
	 * resolved after `normalizedOptions`
	 */
	// @ts-expect-error
	options: ResolvedDevServer;
	// @ts-expect-error
	staticWatchers: FSWatcher[];
	// @ts-expect-error
	sockets: Socket[];
	// @ts-expect-error
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
													// @ts-expect-error
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

		// @ts-expect-error
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
				// enable hot by default
				compiler.options.devServer ??= {};
				compiler.options.devServer.hot = true;
				if (
					// @ts-expect-error
					!compiler.options.experiments.rspackFuture.disableTransformByDefault
				) {
					compiler.options.builtins.react ??= {};
					// enable react.development by default
					compiler.options.builtins.react.development ??= true;
					// enable react.refresh by default
					compiler.options.builtins.react.refresh ??= true;
					if (compiler.options.builtins.react.refresh) {
						const ReactRefreshPlugin = require("@rspack/plugin-react-refresh");
						const runtimePaths = ReactRefreshPlugin.deprecated_runtimePaths;
						new compiler.webpack.EntryPlugin(
							compiler.context,
							runtimePaths[0],
							{
								name: undefined
							}
						).apply(compiler);
						new compiler.webpack.ProvidePlugin({
							$ReactRefreshRuntime$: runtimePaths[1]
						}).apply(compiler);
						compiler.options.module.rules.unshift({
							include: runtimePaths,
							type: "js"
						});
					}
				}
			} else if (compiler.options.builtins.react?.refresh) {
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
				// @ts-expect-error: `addAdditionalEntries` is private function in base class.
				this.addAdditionalEntries(compiler);
				new compiler.webpack.ProvidePlugin({
					__webpack_dev_server_client__: this.getClientTransport()
				}).apply(compiler);
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
					// @ts-expect-error
					middleware: (req, res) => {
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
				// @ts-expect-error
				this.app.use(middleware);
			} else if (typeof middleware.path !== "undefined") {
				// @ts-expect-error
				this.app.use(middleware.path, middleware.middleware);
			} else {
				// @ts-expect-error
				this.app.use(middleware.middleware);
			}
		});

		// @ts-expect-error
		super.setupMiddlewares();
	}
}
