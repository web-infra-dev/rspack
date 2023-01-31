import WebpackDevServer from "webpack-dev-server";

export interface Configuration extends WebpackDevServer.Configuration {
	hot?: boolean;
}

export interface ResolvedConfiguration extends Configuration {
	port: number | string;
	static: false | Array<WebpackDevServer.NormalizedStatic>;
	devMiddleware: Configuration["devMiddleware"];
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
