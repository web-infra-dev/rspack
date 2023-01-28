import type { Dev, RspackOptionsNormalized } from "@rspack/core";
import type { WatchOptions } from "chokidar";
import type { Options as ConnectHistoryApiFallbackOptions } from "connect-history-api-fallback";
import path from "path";
import { ProxyOptions, ClientOptions } from "@rspack/core/src/config/devServer";

type WebSocketServerConfiguration =
	| false
	| {
			type: "sockjs" | "ws" | string | Function;
			options: {
				protocol?: string;
				host?: string;
				port?: number;
				prefix?: string;
				path?: string;
			};
	  };

export interface ResolvedDev {
	host: string;
	port: number | string;
	static: {
		directory: string;
		watch: false | WatchOptions;
	};
	devMiddleware: {};
	hot: boolean;
	open: boolean;
	liveReload: boolean;
	webSocketServer: WebSocketServerConfiguration;
	proxy: ProxyOptions;
	client: ClientOptions;
	historyApiFallback: false | ConnectHistoryApiFallbackOptions;
}

function resolveStaticWatchOptions(watch: WatchOptions = {}) {
	const ignored = watch.ignored ?? [
		"**/dist/**",
		"**/node_modules/**",
		"**/.git/**"
	];
	return {
		...watch,
		ignored
	};
}

export function resolveDevOptions(
	options: Dev,
	compilerOptions: RspackOptionsNormalized
): ResolvedDev {
	const open = true;
	const proxy = options.proxy;
	const hot = options.hot ?? true;
	// --- static
	const directory =
		options.static?.directory ??
		path.resolve(compilerOptions.context, compilerOptions.output.path);
	let watch: false | WatchOptions = {};
	if (options.static?.watch === false) {
		watch = false;
	} else if (options.static?.watch === true) {
		watch = resolveStaticWatchOptions({});
	} else if (options.static?.watch) {
		watch = options.static?.watch;
	}
	// ---
	const devMiddleware = options.devMiddleware ?? {};
	const liveReload = options.liveReload ?? true;

	if (
		typeof options.client === "undefined" ||
		(typeof options.client === "object" && options.client !== null)
	) {
		if (!options.client) {
			options.client = {};
		}

		if (typeof options.client.webSocketURL === "undefined") {
			options.client.webSocketURL = {};
		} else if (typeof options.client.webSocketURL === "string") {
			const parsedURL = new URL(options.client.webSocketURL);
			options.client.webSocketURL = {
				protocol: parsedURL.protocol,
				hostname: parsedURL.hostname,
				port: parsedURL.port.length > 0 ? Number(parsedURL.port) : "",
				pathname: parsedURL.pathname,
				username: parsedURL.username,
				password: parsedURL.password
			};
		} else if (typeof options.client.webSocketURL.port === "string") {
			options.client.webSocketURL.port = Number(
				options.client.webSocketURL.port
			);
		}

		// Enable client overlay by default
		if (typeof options.client.overlay === "undefined") {
			options.client.overlay = true;
		} else if (typeof options.client.overlay !== "boolean") {
			options.client.overlay = {
				errors: true,
				warnings: true,
				...options.client.overlay
			};
		}

		if (typeof options.client.reconnect === "undefined") {
			options.client.reconnect = 10;
		} else if (options.client.reconnect === true) {
			options.client.reconnect = Infinity;
		} else if (options.client.reconnect === false) {
			options.client.reconnect = 0;
		}

		// Respect infrastructureLogging.level
		if (typeof options.client.logging === "undefined") {
			options.client.logging = compilerOptions.infrastructureLogging
				? compilerOptions.infrastructureLogging.level
				: "info";
		}
	}

	const defaultWebSocketServerType = "ws";
	const defaultWebSocketServerOptions = { path: "/ws" };

	let webSocketServer: WebSocketServerConfiguration;

	if (typeof options.webSocketServer === "undefined") {
		webSocketServer = {
			type: defaultWebSocketServerType,
			options: defaultWebSocketServerOptions
		};
	} else if (
		typeof options.webSocketServer === "boolean" &&
		!options.webSocketServer
	) {
		webSocketServer = false;
	} else if (
		typeof options.webSocketServer === "string" ||
		typeof options.webSocketServer === "function"
	) {
		webSocketServer = {
			type: options.webSocketServer,
			options: defaultWebSocketServerOptions
		};
	} else {
		const { port } = options.webSocketServer.options;
		webSocketServer = {
			type: options.webSocketServer.type || defaultWebSocketServerType,
			options: {
				...defaultWebSocketServerOptions,
				...options.webSocketServer.options,
				port: typeof port === "string" ? Number(port) : port
			}
		};
	}

	let historyApiFallback: ResolvedDev["historyApiFallback"];
	if (typeof options.historyApiFallback === "undefined") {
		historyApiFallback = false;
	} else if (
		typeof options.historyApiFallback === "boolean" &&
		options.historyApiFallback
	) {
		historyApiFallback = {};
	}

	return {
		host: options.host,
		port: options.port ? Number(options.port) : undefined,
		static: {
			directory,
			watch
		},
		devMiddleware,
		open,
		hot,
		liveReload,
		webSocketServer,
		proxy,
		client: options.client,
		historyApiFallback
	};
}
