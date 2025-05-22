/**
 * `RSPACK_PROFILE=ALL` // all trace events
 * `RSPACK_PROFILE=OVERVIEW` // overview trace events
 * `RSPACK_PROFILE=warn,tokio::net=info` // trace filter from  https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#example-syntax
 */
import fs from "node:fs";
import path from "node:path";
import { rspack } from "@rspack/core";

const overviewTraceFilter = "info";
const allTraceFilter = "trace";
const defaultRustTraceLayer = "chrome";

enum TracePreset {
	OVERVIEW = "OVERVIEW", // contains overview trace events
	ALL = "ALL" // contains all trace events
}

function resolveLayer(value: string): string {
	if (value === TracePreset.OVERVIEW) {
		return overviewTraceFilter;
	}
	if (value === TracePreset.ALL) {
		return allTraceFilter;
	}
	return value;
}

export async function applyProfile(
	filterValue: string,
	traceLayer: string = defaultRustTraceLayer,
	traceOutput?: string
) {
	const { asyncExitHook } = await import("exit-hook");

	if (traceLayer !== "chrome" && traceLayer !== "logger") {
		throw new Error(`unsupported trace layer: ${traceLayer}`);
	}

	if (!traceOutput) {
		const timestamp = Date.now();
		const defaultOutputDir = path.resolve(
			`.rspack-profile-${timestamp}-${process.pid}`
		);
		const defaultRustTraceChromeOutput = path.join(
			defaultOutputDir,
			"trace.json"
		);
		const defaultRustTraceLoggerOutput = "stdout";

		const defaultTraceOutput =
			traceLayer === "chrome"
				? defaultRustTraceChromeOutput
				: defaultRustTraceLoggerOutput;

		// biome-ignore lint/style/noParameterAssign: setting default value makes sense
		traceOutput = defaultTraceOutput;
	}

	const filter = resolveLayer(filterValue);

	await ensureFileDir(traceOutput);
	await rspack.experiments.globalTrace.register(
		filter,
		traceLayer,
		traceOutput
	);
	asyncExitHook(rspack.experiments.globalTrace.cleanup, {
		wait: 500
	});
}

async function ensureFileDir(outputFilePath: string) {
	const dir = path.dirname(outputFilePath);
	await fs.promises.mkdir(dir, { recursive: true });
}
