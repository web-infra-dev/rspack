import type { WatchOptions } from "chokidar";
import type {
	Options as HttpProxyMiddlewareOptions,
	Filter as HttpProxyMiddlewareOptionsFilter
} from "http-proxy-middleware";
export interface WebSocketServerOptions {
	protocol?: string;
	host?: string;
	port?: number;
	prefix?: string;
	path?: string;
}
type Bypass = (req: Request, res: Response, proxyConfig: ProxyOptions) => void;
export type ProxyOptions = HttpProxyMiddlewareOptions & { bypass?: Bypass } & {
	context?: HttpProxyMiddlewareOptionsFilter | undefined;
	path?: HttpProxyMiddlewareOptionsFilter | undefined;
};
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
