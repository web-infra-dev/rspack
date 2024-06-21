/*
The full syntax, remember update this when you change something in this file.

`RSPACK_PROFILE='TRACE=filter=trace&output=./rspack.trace&layer=chrome|JSCPU=output=./rspack.jscpuprofile|LOGGING=output=./rspack.logging' rspack build`
											 ^----------------------------------------------: querystring syntax trace options
																																			^: | is a delimiter for different profile options
																																			 ^---------------------------------: querystring syntax js cpuprofile options
																																																				 ^: | is a delimiter for different profile options
																																																					^------------------------------: querystring syntax stats.logging options
											 ^-----------: trace filter, default to `trace`, more syntax: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#example-syntax
																		^--------------------: trace output, `stderr`, `stdout`, or a file path, default to `./.rspack-profile-${timestamp}/trace.json` for layer `chrome` and default to `stdout` for layer `logger`
																													^-----------: trace layer, `chrome` or `logger`, default to `chrome`
																																						 ^---------------------------: js cpuprofile output, `stderr`, `stdout`, or a file path, default to `./.rspack-profile-${timestamp}/jscpuprofile.json`
																																																									^----------------------: stats.logging output, default to `./.rspack-profile-${timestamp}/logging.json`

`RSPACK_PROFILE='TRACE=filter=trace&output=./rspack.trace&layer=chrome' rspack build`: only enable trace

`RSPACK_PROFILE=TRACE rspack build`: only enable trace, and use default options for trace

`RSPACK_PROFILE='JSCPU=output=./rspack.jscpuprofile' rspack build`: only enable js cpuprofile

`RSPACK_PROFILE=JSCPU rspack build`: only enable js cpuprofile, and use default options for js cpuprofile

`RSPACK_PROFILE='LOGGING=output=./rspack.logging' rspack build`: only enable stats.logging

`RSPACK_PROFILE=LOGGING rspack build`: only enable stats.logging, and use default options for stats.logging

`RSPACK_PROFILE=ALL rspack build`: enable all, and use default options

`RSPACK_PROFILE=[rspack_node,rspack_core] rspack build`: enable all, but customize trace filter

*/

import fs from "fs";
import path from "path";
import { URLSearchParams } from "url";
import {
	type Compiler,
	RspackOptions,
	experimental_cleanupGlobalTrace as cleanupGlobalTrace,
	experimental_registerGlobalTrace as registerGlobalTrace
} from "@rspack/core";
import inspector from "inspector";

type JSCPUProfileOptionsOutput = string;
type JSCPUProfileOptions = {
	output: JSCPUProfileOptionsOutput;
};
type ParametersOfRegisterGlobalTrace = Parameters<typeof registerGlobalTrace>;
type RustTraceOptionsFilter = ParametersOfRegisterGlobalTrace[0];
type RustTraceOptionsLayer = ParametersOfRegisterGlobalTrace[1];
type RustTraceOptionsOutput = ParametersOfRegisterGlobalTrace[2];
type RustTraceOptions = {
	filter: RustTraceOptionsFilter;
	layer: RustTraceOptionsLayer;
	output: RustTraceOptionsOutput;
};
type LoggingOutputOptions = string;
type LoggingOptions = {
	output: LoggingOutputOptions;
};
type ProfileOptions = {
	TRACE?: RustTraceOptions;
	JSCPU?: JSCPUProfileOptions;
	LOGGING?: LoggingOptions;
};

const timestamp = Date.now();
const defaultOutputDirname = path.resolve(
	`.rspack-profile-${timestamp}-${process.pid}`
);
const defaultJSCPUProfileOutput = path.join(
	defaultOutputDirname,
	`./jscpuprofile.json`
);
const defaultRustTraceChromeOutput = path.join(
	defaultOutputDirname,
	`./trace.json`
);
const defaultRustTraceLoggerOutput = `stdout`;
const defaultRustTraceFilter = "trace";
const defaultRustTraceLayer = "chrome";
const defaultLoggingOutput = path.join(defaultOutputDirname, `./logging.json`);

function resolveProfile(value: string): ProfileOptions {
	if (value.toUpperCase() === "ALL") {
		return {
			TRACE: {
				filter: defaultRustTraceFilter,
				layer: defaultRustTraceLayer,
				output: defaultRustTraceChromeOutput
			},
			JSCPU: { output: defaultJSCPUProfileOutput },
			LOGGING: { output: defaultLoggingOutput }
		};
	}
	if (value.startsWith("[") && value.endsWith("]")) {
		return {
			TRACE: resolveRustTraceOptions(value.slice(1, value.length - 1)),
			JSCPU: { output: defaultJSCPUProfileOutput },
			LOGGING: { output: defaultLoggingOutput }
		};
	}
	return value.split("|").reduce<ProfileOptions>((acc, cur) => {
		const upperCur = cur.toUpperCase();
		if (upperCur.startsWith("TRACE")) {
			acc.TRACE = resolveRustTraceOptions(cur.slice(6));
		} else if (upperCur.startsWith("JSCPU")) {
			acc.JSCPU = resolveJSCPUProfileOptions(cur.slice(6));
		} else if (upperCur.startsWith("LOGGING")) {
			acc.LOGGING = resolveLoggingOptions(cur.slice(8));
		}
		return acc;
	}, {});
}

// JSCPU=value
function resolveJSCPUProfileOptions(value: string): JSCPUProfileOptions {
	// output=filepath
	if (value.includes("=")) {
		const parsed = new URLSearchParams(value);
		return { output: parsed.get("output") || defaultJSCPUProfileOutput };
	}
	// filepath
	return { output: value || defaultJSCPUProfileOutput };
}

// TRACE=value
function resolveRustTraceOptions(value: string): RustTraceOptions {
	// filter=trace&output=stdout&layer=logger
	if (value.includes("=")) {
		const parsed = new URLSearchParams(value);
		const filter = parsed.get("filter") || defaultRustTraceFilter;
		const layer = parsed.get("layer") || defaultRustTraceLayer;
		const output =
			layer === "chrome"
				? parsed.get("output") || defaultRustTraceChromeOutput
				: parsed.get("output") || defaultRustTraceLoggerOutput;
		if (layer !== "chrome" && layer !== "logger") {
			throw new Error(
				`${layer} is not a valid layer, should be chrome or logger`
			);
		}
		return {
			filter,
			layer,
			output
		};
	}
	// trace
	return {
		filter: value || defaultRustTraceFilter,
		layer: defaultRustTraceLayer,
		output: defaultRustTraceChromeOutput
	};
}

// LOGGING=value
function resolveLoggingOptions(value: string): LoggingOptions {
	// output=filepath
	if (value.includes("=")) {
		const parsed = new URLSearchParams(value);
		return { output: parsed.get("output") || defaultLoggingOutput };
	}
	// filepath
	return { output: value || defaultLoggingOutput };
}

class RspackProfileJSCPUProfilePlugin {
	constructor(private output: string) {}

	apply(compiler: Compiler) {
		const session = new inspector.Session();
		session.connect();
		session.post("Profiler.enable");
		session.post("Profiler.start");
		compiler.hooks.done.tapAsync(
			RspackProfileJSCPUProfilePlugin.name,
			(_stats, callback) => {
				if (compiler.watchMode) return callback();
				session.post("Profiler.stop", (error, param) => {
					if (error) {
						console.error("Failed to generate JS CPU profile:", error);
						return;
					}
					fs.writeFileSync(this.output, JSON.stringify(param.profile));
				});
				return callback();
			}
		);
	}
}

class RspackProfileLoggingPlugin {
	constructor(private output: string) {}

	apply(compiler: Compiler) {
		compiler.hooks.done.tapAsync(
			RspackProfileLoggingPlugin.name,
			(stats, callback) => {
				if (compiler.watchMode) return callback();
				const logging = stats.toJson({
					all: false,
					logging: "verbose",
					loggingTrace: true
				});
				fs.writeFileSync(this.output, JSON.stringify(logging));
				return callback();
			}
		);
	}
}

export async function applyProfile(profileValue: string, item: RspackOptions) {
	const { default: exitHook } = await import("exit-hook");
	const entries = Object.entries(resolveProfile(profileValue));
	if (entries.length <= 0) return;
	for (const [kind, value] of entries) {
		await ensureFileDir(value.output);
		if (kind === "TRACE" && "filter" in value) {
			registerGlobalTrace(value.filter, value.layer, value.output);
			exitHook(cleanupGlobalTrace);
		} else if (kind === "JSCPU") {
			(item.plugins ??= []).push(
				new RspackProfileJSCPUProfilePlugin(value.output)
			);
		} else if (kind === "LOGGING") {
			(item.plugins ??= []).push(new RspackProfileLoggingPlugin(value.output));
		}
	}
}

async function ensureFileDir(outputFilePath: string) {
	const dir = path.dirname(outputFilePath);
	await fs.promises.mkdir(dir, { recursive: true });
	return dir;
}
