import type { RspackCLI } from "./cli";

export type { Configuration } from "@rspack/core";

export type LogHandler = (value: any) => void;

export type RspackCLIColors = {
	isColorSupported: boolean;
} & Omit<typeof import("picocolors"), "createColors">;

export interface RspackCLILogger {
	error: LogHandler;
	warn: LogHandler;
	info: LogHandler;
	success: LogHandler;
	log: LogHandler;
	raw: LogHandler;
}

export interface RspackCommand {
	apply(cli: RspackCLI): Promise<void>;
}
