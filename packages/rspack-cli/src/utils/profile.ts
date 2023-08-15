import inspector from "inspector";
import fs from "fs";
import { URLSearchParams } from "url";
import { experimental_registerGlobalTrace as registerGlobalTrace } from "@rspack/core";

type JSCPUProfileOptionsOutput = string;
export type JSCPUProfileOptions = {
	output: JSCPUProfileOptionsOutput;
};
type ParametersOfRegisterGlobalTrace = Parameters<typeof registerGlobalTrace>;
type RustTraceOptionsFilter = ParametersOfRegisterGlobalTrace[0];
type RustTraceOptionsLayer = ParametersOfRegisterGlobalTrace[1];
type RustTraceOptionsOutput = ParametersOfRegisterGlobalTrace[2];
export type RustTraceOptions = {
	filter: RustTraceOptionsFilter;
	layer: RustTraceOptionsLayer;
	output: RustTraceOptionsOutput;
};
type LoggingOutputOptions = string;
export type LoggingOptions = {
	output: LoggingOutputOptions;
};
export type ProfileOptions = {
	TRACE?: RustTraceOptions;
	JSCPU?: JSCPUProfileOptions;
	LOGGING?: LoggingOptions;
};

const defaultJSCPUProfileOutput = `./rspack.jscpuprofile`;
const defaultRustTraceChromeOutput = `./rspack.trace`;
const defaultRustTraceLoggerOutput = `stdout`;
const defaultRustTraceFilter = "trace";
const defaultRustTraceLayer = "chrome";
const defaultLoggingOutput = `./rspack.logging`;

export function resolveProfile(value: string): ProfileOptions {
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
		if (cur.toUpperCase().startsWith("TRACE")) {
			acc.TRACE = resolveRustTraceOptions(cur.slice(6));
		}
		if (cur.toUpperCase().startsWith("JSCPU")) {
			acc.JSCPU = resolveJSCPUProfileOptions(cur.slice(6));
		}
		if (cur.toUpperCase().startsWith("LOGGING")) {
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

export class RspackProfileJSCPUProfilePlugin {
	constructor(private output: string) {}

	apply(compiler) {
		const session = new inspector.Session();
		session.connect();
		session.post("Profiler.enable");
		session.post("Profiler.start");
		compiler.hooks.done.tapAsync(
			RspackProfileJSCPUProfilePlugin.name,
			(stats, callback) => {
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

export class RspackProfileLoggingPlugin {
	constructor(private output: string) {}

	apply(compiler) {
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
