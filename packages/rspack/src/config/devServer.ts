import path from "node:path";
import { resolveWatchOption } from "./watch";
import type { WatchOptions } from "chokidar";

export interface WebSocketServerOptions {
	protocol?: string;
	host?: string;
	port?: number;
	prefix?: string;
	path?: string;
}

export interface Dev {
	port?: number;
	// TODO: static maybe `boolean`, `string`, `object`, `array`
	static?: {
		directory?: string;
		watch?: boolean | WatchOptions;
	};
	devMiddleware?: {};
	hot?: boolean;
	open?: boolean;
	liveReload?: boolean;
	// TODO: only support ws.
	webSocketServer?: boolean | WebSocketServerOptions;
}

export interface ResolvedDev {
	port: number;
	static: {
		directory: string;
		watch: false | WatchOptions;
	};
	devMiddleware: {};
	hot: boolean;
	open: boolean;
	liveReload: boolean;
	webSocketServer: false | WebSocketServerOptions;
}

export function getAdditionDevEntry() {
	const devClientEntryPath = require.resolve("@rspack/dev-client");
	const additionalEntry = {
		"rspack-dev-client": devClientEntryPath,
		"rspack-hot-update": require.resolve("@rspack/dev-client/devServer")
	};
	// console.log(additionalEntry);
	return additionalEntry;
}

interface ResolveDevOptionContext {
	context: string;
}

export function resolveDevOptions(
	devConfig: Dev = {},
	context: ResolveDevOptionContext
): ResolvedDev {
	const port = devConfig.port ?? 8080;
	const open = true;
	const hot = devConfig.hot ?? false;
	// --- static
	const directory =
		devConfig.static?.directory ?? path.resolve(context.context, "dist");
	let watch: false | WatchOptions = {};
	if (devConfig.static?.watch === false) {
		watch = false;
	} else if (devConfig.static?.watch === true) {
		watch = resolveWatchOption({});
	} else if (devConfig.static?.watch) {
		watch = devConfig.static?.watch;
	}
	// ---
	const devMiddleware = devConfig.devMiddleware ?? {};
	const liveReload = devConfig.liveReload ?? true;

	let webSocketServer: false | WebSocketServerOptions = {};
	if (devConfig.webSocketServer === false) {
		webSocketServer = false;
	} else if (devConfig.webSocketServer === true) {
		webSocketServer = {};
	} else if (devConfig.webSocketServer) {
		webSocketServer = devConfig.webSocketServer;
	}

	return {
		port,
		static: {
			directory,
			watch
		},
		devMiddleware,
		open,
		hot,
		liveReload,
		webSocketServer
	};
}
