import WebpackDevServer from "webpack-dev-server";

export interface DevServer extends WebpackDevServer.Configuration {
	hot?: boolean;
}
