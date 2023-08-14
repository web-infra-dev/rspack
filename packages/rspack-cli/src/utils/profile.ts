import inspector from "inspector";
import fs from "fs";
import { URLSearchParams } from "url";
import { registerGlobalTrace } from "@rspack/core";

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
export type ProfileOptions = {
	TRACE?: RustTraceOptions;
	JSCPU?: JSCPUProfileOptions;
};

const defaultJSCPUProfileOutput = `./rspack.jscpuprofile`;
const defaultRustTraceChromeOutput = `./rspack.trace`;
const defaultRustTraceLoggerOutput = `stdout`;
const defaultRustTraceFilter = "trace";
const defaultRustTraceLayer = "chrome";

export function resolveProfile(value: string): ProfileOptions {
	if (value.toUpperCase() === "ALL") {
		return {
			TRACE: {
				filter: defaultRustTraceFilter,
				layer: defaultRustTraceLayer,
				output: defaultRustTraceChromeOutput
			},
			JSCPU: { output: defaultJSCPUProfileOutput }
		};
	}
	if (value.startsWith("[") && value.endsWith("]")) {
		return {
			TRACE: resolveRustTraceOptions(value.slice(1, value.length - 1)),
			JSCPU: { output: defaultJSCPUProfileOutput }
		};
	}
	return value.split("|").reduce<ProfileOptions>((acc, cur) => {
		if (cur.toUpperCase().startsWith("TRACE")) {
			acc.TRACE = resolveRustTraceOptions(cur.slice(6));
		}
		if (cur.toUpperCase().startsWith("JSCPU")) {
			acc.JSCPU = resolveJSCPUProfileOptions(cur.slice(6));
		}
		return acc;
	}, {});
}

// JSCPU=value
function resolveJSCPUProfileOptions(value: string): JSCPUProfileOptions {
	// output=stderr
	if (value.includes("=")) {
		const parsed = new URLSearchParams(value);
		return { output: parsed.get("output") || defaultJSCPUProfileOutput };
	}
	// stderr
	return { output: value || defaultJSCPUProfileOutput };
}

// TRACE=value
function resolveRustTraceOptions(value: string): RustTraceOptions {
	// filter=trace&output=stderr&layer=logger
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

export class RspackJsCPUProfilePlugin {
	constructor(private output: string) {}

	apply(compiler) {
		const session = new inspector.Session();
		session.connect();
		session.post("Profiler.enable");
		session.post("Profiler.start");
		compiler.hooks.done.tapAsync(
			RspackJsCPUProfilePlugin.name,
			(stats, callback) => {
				if (compiler.watchMode) return callback();
				session.post("Profiler.stop", (error, param) => {
					if (error) {
						console.error("Failed to generate JS CPU profile:", error);
						return;
					}
					fs.writeFileSync(this.output, JSON.stringify(param.profile));
				});
			}
		);
	}
}
