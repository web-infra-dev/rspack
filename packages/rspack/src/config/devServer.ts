import type { WatchOptions } from "chokidar";

export interface WebSocketServerOptions {
	protocol?: string;
	host?: string;
	port?: number;
	prefix?: string;
	path?: string;
}
export type ProxyOptions = any;
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
	// TODO: only support ws.
	webSocketServer?: boolean | WebSocketServerOptions;
}
