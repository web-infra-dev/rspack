import { Colorette } from "colorette";
import { RspackCLI } from "./rspack-cli";
import { WebpackOptionsNormalized } from "webpack";
import { Configuration as DevServerConfig } from "webpack-dev-server";
export interface IRspackCLI {
	runRspack(): Promise<void>;
}
export type LogHandler = (value: any) => void;
export interface RspackCLIColors extends Colorette {
	isColorSupported: boolean;
}
export interface RspackCLILogger {
	error: LogHandler;
	warn: LogHandler;
	info: LogHandler;
	success: LogHandler;
	log: LogHandler;
	raw: LogHandler;
}

export interface RspackCLIOptions {
	entry?: string[];
	config?: string;
	devtool?: boolean;
	mode?: string;
	watch?: boolean;
	analyze?: boolean;
}

export interface RspackCommand {
	apply(cli: RspackCLI): Promise<void>;
}
export type RspackDevServerOptions = DevServerConfig & WebpackOptionsNormalized;
