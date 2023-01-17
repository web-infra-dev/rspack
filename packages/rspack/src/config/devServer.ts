import WebpackDevServer from "webpack-dev-server";

export interface Dev extends WebpackDevServer.Configuration {
	hot?: boolean;
}
