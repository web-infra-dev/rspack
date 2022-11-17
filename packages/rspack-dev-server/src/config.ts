import type {
	Dev,
	WebSocketServerOptions,
	RspackOptionsNormalized
} from "@rspack/core";
import type { WatchOptions } from "chokidar";
import path from "node:path";
import { resolveWatchOption } from "@rspack/core";

export interface ResolvedDev {
	host: string;
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

export function resolveDevOptions(
	devConfig: Dev,
	compilerOptions: RspackOptionsNormalized
): ResolvedDev {
	const open = true;
	const hot = devConfig.hot ?? true;
	// --- static
	const directory =
		devConfig.static?.directory ??
		path.resolve(compilerOptions.context, "dist");
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
		host: devConfig.host,
		port: devConfig.port ? Number(devConfig.port) : undefined,
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
