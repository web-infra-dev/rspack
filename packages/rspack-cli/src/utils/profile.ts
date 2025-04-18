/*
The full syntax, remember update this when you change something in this file.

`RSPACK_PROFILE='TRACE=filter=trace&output=./rspack.trace&layer=chrome
											 ^----------------------------------------------: querystring syntax trace options
																																			^: | is a delimiter for different profile options
																																			 ^---------------------------------: querystring syntax js cpuprofile options
																																																				 ^: | is a delimiter for different profile options
																																																					^------------------------------: querystring syntax stats.logging options
											 ^-----------: trace filter, default to `trace`, more syntax: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#example-syntax
																		^--------------------: trace output, `stderr`, `stdout`, or a file path, default to `./.rspack-profile-${timestamp}/trace.json` for layer `chrome` and default to `stdout` for layer `logger`
																													^-----------: trace layer, `chrome` or `logger`, default to `chrome`

`RSPACK_PROFILE='TRACE=filter=trace&output=./rspack.trace&layer=chrome' rspack build`: only enable trace

`RSPACK_PROFILE=TRACE rspack build`: only enable trace, and use default options for trace
`RSPACK_PROFILE=ALL rspack build`: enable all, and use default options

`RSPACK_PROFILE=[rspack_node,rspack_core] rspack build`: enable all, but customize trace filter

*/

import fs from "node:fs";
import path from "node:path";
import { URLSearchParams } from "node:url";
import { type RspackOptions, rspack } from "@rspack/core";

type ParametersOfRegisterGlobalTrace = Parameters<
	typeof rspack.experiments.globalTrace.register
>;
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
	LOGGING?: LoggingOptions;
};

const timestamp = Date.now();
const defaultOutputDirname = path.resolve(
	`.rspack-profile-${timestamp}-${process.pid}`
);
const defaultRustTraceChromeOutput = path.join(
	defaultOutputDirname,
	"./trace.json"
);
const defaultRustTraceLoggerOutput = "stdout";
const defaultRustTraceFilter = "info";
const defaultRustTraceLayer = "chrome";

function resolveProfile(value: string): ProfileOptions {
	if (value.toUpperCase() === "ALL") {
		return {
			TRACE: {
				filter: defaultRustTraceFilter,
				layer: defaultRustTraceLayer,
				output: defaultRustTraceChromeOutput
			}
		};
	}
	if (value.startsWith("[") && value.endsWith("]")) {
		return {
			TRACE: resolveRustTraceOptions(value.slice(1, value.length - 1))
		};
	}
	return value.split("|").reduce<ProfileOptions>((acc, cur) => {
		const upperCur = cur.toUpperCase();
		if (upperCur.startsWith("TRACE")) {
			acc.TRACE = resolveRustTraceOptions(cur.slice(6));
		}
		return acc;
	}, {});
}

function isSupportedLayer(layer: string): layer is RustTraceOptionsLayer {
	const SUPPORTED_LAYERS = ["chrome", "logger"];
	return SUPPORTED_LAYERS.includes(layer);
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

		if (!isSupportedLayer(layer)) {
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

export async function applyProfile(profileValue: string, item: RspackOptions) {
	const { asyncExitHook } = await import("exit-hook");
	const entries = Object.entries(resolveProfile(profileValue));
	if (entries.length <= 0) return;
	await fs.promises.mkdir(defaultOutputDirname);
	for (const [kind, value] of entries) {
		await ensureFileDir(value.output);
		if (kind === "TRACE" && "filter" in value) {
			await rspack.experiments.globalTrace.register(
				value.filter,
				value.layer,
				value.output
			);
			asyncExitHook(rspack.experiments.globalTrace.cleanup, {
				wait: 500
			});
		}
	}
}

async function ensureFileDir(outputFilePath: string) {
	const dir = path.dirname(outputFilePath);
	await fs.promises.mkdir(dir, { recursive: true });
	return dir;
}
