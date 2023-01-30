import type {Configuration as WebpackDevServerConfiguration} from "webpack-dev-server";

export interface Dev extends WebpackDevServerConfiguration {
	hot?: boolean;
}
