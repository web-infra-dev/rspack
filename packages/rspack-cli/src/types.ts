import { Colorette } from "colorette";
import { RspackCLI } from "./rspack-cli";
import type { DevServer } from "@rspack/core";
export type { Configuration } from "@rspack/core";

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
	config?: string;
	argv?: Record<string, any>;
	configName?: string[];
}

export interface RspackCLIBuildOptions extends RspackCLIOptions {
	entry?: string[];
	devtool?: boolean;
	mode?: string;
	watch?: boolean;
	analyze?: boolean;
	env?: Record<string, any>;
	nodeEnv: string;
}
export interface RspackCLIPreviewOptions extends RspackCLIOptions {
	dir?: string;
	port?: number;
	host?: string;
	open?: boolean;
	server?: string;
	publicPath: string;
}

export interface RspackCommand {
	apply(cli: RspackCLI): Promise<void>;
}
export type RspackDevServerOptions = DevServer;
