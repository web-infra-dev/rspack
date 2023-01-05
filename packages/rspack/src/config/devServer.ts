import type { WatchOptions } from "chokidar";
import WebpackDevServer from "webpack-dev-server";

export interface WebSocketServerOptions {
	protocol?: string;
	host?: string;
	port?: number | string;
	prefix?: string;
	path?: string;
}

export type ProxyOptions =
	| WebpackDevServer.ProxyConfigMap
	| WebpackDevServer.ProxyConfigArrayItem
	| WebpackDevServer.ProxyConfigArray;
export type ClientOptions = WebpackDevServer.ClientConfiguration | boolean;

export interface Dev {
	host?: string;
	port?: number | string;
	// TODO: static maybe `boolean`, `string`, `object`, `array`
	static?: {
		directory?: string;
		watch?: boolean | WatchOptions;
	};
	proxy?: ProxyOptions;
	devMiddleware?: {};
	hot?: boolean;
	open?: boolean;
	liveReload?: boolean;
	webSocketServer?:
		| false
		| "sockjs"
		| "ws"
		| {
				type?: "sockjs" | "ws" | string | Function;
				options?: WebSocketServerOptions;
		  }
		| Function;
	client?: ClientOptions;
}
