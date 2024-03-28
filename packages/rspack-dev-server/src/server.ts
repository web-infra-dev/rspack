/**
 * The following code is modified based on
 * https://github.com/webpack/webpack-dev-server/blob/b0f15ace0123c125d5870609ef4691c141a6d187/lib/Server.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack-dev-server/blob/b0f15ace0123c125d5870609ef4691c141a6d187/LICENSE
 */
import path from "node:path";
import { Compiler, MultiCompiler } from "@rspack/core";
import type { Socket } from "net";
import type { FSWatcher } from "chokidar";
import rdm from "webpack-dev-middleware";
import type { Server } from "http";
import fs from "fs";
import WebpackDevServer from "webpack-dev-server";
import type { ResolvedDevServer, DevServer } from "./config";
import { applyDevServerPatch } from "./patch";

applyDevServerPatch();

export class RspackDevServer extends WebpackDevServer {
	/**
	 * resolved after `normalizedOptions`
	 */
	declare options: ResolvedDevServer;

	declare staticWatchers: FSWatcher[];

	declare sockets: Socket[];

	declare server: Server;
	// TODO: remove @ts-ignore here
	/** @ts-ignore */
	public compiler: Compiler | MultiCompiler;
	webSocketServer: WebpackDevServer.WebSocketServerImplementation | undefined;

	constructor(options: DevServer, compiler: Compiler | MultiCompiler) {
		super(options, compiler as any);
	}

	private override getClientTransport(): string {
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

	override async initialize() {
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

				const HMRPluginExists = compiler.options.plugins.find(
					p => p?.constructor === compiler.webpack.HotModuleReplacementPlugin
				);

				if (HMRPluginExists) {
					this.logger.warn(
						`"hot: true" automatically applies HMR plugin, you don't have to add it manually to your webpack configuration.`
					);
				} else {
					// Apply the HMR plugin
					const plugin = new compiler.webpack.HotModuleReplacementPlugin();

					plugin.apply(compiler);
				}

				// Apply modified version of `ansi-html-community`
				compiler.options.resolve.alias = {
					"ansi-html-community": path.resolve(__dirname, "./ansiHTML"),
					...compiler.options.resolve.alias
				};
			}
		});

		if (this.options.webSocketServer) {
			compilers.forEach(compiler => {
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

	private override setupDevMiddleware() {
		// @ts-expect-error
		this.middleware = rdm(this.compiler, this.options.devMiddleware);
	}

	private override setupMiddlewares() {
		const middlewares: WebpackDevServer.Middleware[] = [];
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

	private override addAdditionalEntries(compiler: Compiler) {
		const additionalEntries = [];
		// @ts-expect-error
		const isWebTarget = WebpackDevServer.isWebTarget(compiler);

		// TODO maybe empty client
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
					typeof webSocketServer.options?.host !== "undefined" &&
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
					typeof webSocketServer.options?.port !== "undefined" &&
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
					typeof webSocketServer.options?.prefix !== "undefined" ||
					typeof webSocketServer.options?.path !== "undefined"
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
					"@rspack/dev-server/client/index"
				)}?${webSocketURLStr}`
			);
		}

		if (this.options.hot === "only") {
			additionalEntries.push(
				require.resolve("@rspack/core/hot/only-dev-server")
			);
		} else if (this.options.hot) {
			additionalEntries.push(require.resolve("@rspack/core/hot/dev-server"));
		}

		const webpack = compiler.webpack;

		for (const additionalEntry of additionalEntries) {
			new webpack.EntryPlugin(compiler.context, additionalEntry, {
				name: undefined
			}).apply(compiler);
		}
	}
}
