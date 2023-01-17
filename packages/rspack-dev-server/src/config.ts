import type WebpackDevServer from "webpack-dev-server";
import { Dev } from "@rspack/core/src/config/devServer";

export interface ResolvedDev extends Dev {
	port: number | string;
	static: false | Array<WebpackDevServer.NormalizedStatic>;
	devMiddleware: WebpackDevServer.Configuration["devMiddleware"];
	// FIXME: hot should be `boolean | 'only'`
	hot: boolean;
	open: WebpackDevServer.Open[];
	magicHtml: boolean;
	liveReload: boolean;
	webSocketServer: false | WebpackDevServer.WebSocketServerConfiguration;
	proxy: WebpackDevServer.ProxyConfigArray;
	client: WebpackDevServer.Configuration["client"];
	allowedHosts: "auto" | string[] | "all";
	bonjour: false | Record<string, never> | WebpackDevServer.BonjourOptions;
	compress: boolean;
	historyApiFallback: false | WebpackDevServer.ConnectHistoryApiFallbackOptions;
	server: WebpackDevServer.ServerConfiguration;
	ipc: string | undefined;
	setupExitSignals: boolean;
	watchFiles: WebpackDevServer.WatchFiles[];
}
