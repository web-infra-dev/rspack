import type { DevServer } from "@rspack/core";
import WebpackDevServer from "webpack-dev-server";

export type { DevServer };

export interface ResolvedDevServer extends DevServer {
	port: number | string;
	static: false | Array<WebpackDevServer.NormalizedStatic>;
	devMiddleware: DevServer["devMiddleware"];
	hot: boolean | "only";
	host?: string;
	open: WebpackDevServer.Open[];
	magicHtml: boolean;
	liveReload: boolean;
	webSocketServer: false | WebpackDevServer.WebSocketServerConfiguration;
	proxy: WebpackDevServer.ProxyConfigArray;
	client: WebpackDevServer.ClientConfiguration;
	allowedHosts: "auto" | string[] | "all";
	bonjour: false | Record<string, never> | WebpackDevServer.BonjourOptions;
	compress: boolean;
	historyApiFallback: false | WebpackDevServer.ConnectHistoryApiFallbackOptions;
	server: WebpackDevServer.ServerConfiguration;
	ipc: string | undefined;
	setupExitSignals: boolean;
	watchFiles: WebpackDevServer.WatchFiles[];
}
