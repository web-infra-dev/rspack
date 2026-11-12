import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

// The JavaScript tracer records nanosecond offsets from its start time, while
// the native timestamp baseline is microseconds. Treating the offset as
// microseconds moves every record `offset_ms` seconds into the future (about
// 30s for the probe below), so a one second window is enough to catch it.
const CAPTURE_WINDOW_TOLERANCE_MS = 1000;

// A dedicated process keeps the one-shot global trace state away from other
// compiler cases.
const PROBE_SCRIPT = String.raw`
const fs = require("node:fs");
const path = require("node:path");

const core = require(process.argv[1]);
const traceFile = process.argv[2];
const workDir = process.argv[3];

const wait = ms => new Promise(resolve => setTimeout(resolve, ms));
const parseArg = value => {
	try {
		return JSON.parse(value);
	} catch {
		return value;
	}
};

(async () => {
	const captureStart = Date.now();
	await core.experiments.globalTrace.register("OVERVIEW", "logger", traceFile);
	// Keep the probe away from registration so a unit error is unmistakable.
	await wait(25);

	const probe = {};
	const compiler = core.rspack({
		mode: "development",
		context: workDir,
		entry: "./index.js",
		target: "node",
		devtool: false,
		cache: false,
		output: { path: path.join(workDir, "dist"), filename: "main.js" },
		plugins: [
			{
				apply(compiler) {
					compiler.hooks.beforeCompile.tapPromise("TimestampProbe", async () => {
						const start = process.hrtime.bigint();
						await wait(20);
						probe.durationMs = Number(process.hrtime.bigint() - start) / 1e6;
					});
				}
			}
		]
	});

	try {
		await new Promise((resolve, reject) => {
			compiler.run((error, stats) => {
				const statsError = !error && stats && stats.hasErrors()
					? new Error(stats.toString({ all: false, errors: true, errorDetails: true }))
					: null;
				const finalError = error || statsError;
				if (finalError) {
					reject(finalError);
				} else {
					resolve();
				}
			});
		});
	} finally {
		await new Promise((resolve, reject) =>
			compiler.close(error => (error ? reject(error) : resolve()))
		);
	}

	await core.experiments.globalTrace.cleanup();
	const captureEnd = Date.now();

	const records = fs.readFileSync(traceFile, "utf8").trim().split("\n").map(line => JSON.parse(line));
	const record = records.find(entry =>
		entry.target === "javascript" &&
		parseArg(entry.fields?.pluginName) === "TimestampProbe" &&
		parseArg(entry.fields?.hookName) === "beforeCompile"
	);
	if (!record) {
		throw new Error("no logger record for the JavaScript beforeCompile probe");
	}

	process.stdout.write(JSON.stringify({
		captureStart,
		captureEnd,
		timestamp: record.timestamp,
		recordedDurationMs: Number(/^([\d.]+)ms$/.exec(record.fields["time.busy"] || "")?.[1]),
		measuredDurationMs: probe.durationMs
	}));
})().catch(error => {
	console.error(error);
	process.exit(1);
});
`;

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
export default {
	description: "keep JavaScript trace timestamps inside the capture window",
	async run() {
		const root = fs.mkdtempSync(path.join(os.tmpdir(), "rspack-trace-timestamp-"));
		try {
			const workDir = path.join(root, "work");
			fs.mkdirSync(workDir, { recursive: true });
			fs.writeFileSync(path.join(workDir, "index.js"), "module.exports = 42;\n");

			const raw = execFileSync(
				process.execPath,
				[
					"-e",
					PROBE_SCRIPT,
					fileURLToPath(import.meta.resolve("@rspack/core")),
					path.join(root, "rspack.log"),
					workDir
				],
				{
					stdio: "pipe",
					windowsHide: true,
					timeout: 20_000
				}
			);
			const lines = raw.toString().trim().split("\n");
			const result = JSON.parse(lines[lines.length - 1]);
			const timestamp = Date.parse(result.timestamp);

			expect(timestamp).toBeGreaterThanOrEqual(result.captureStart - CAPTURE_WINDOW_TOLERANCE_MS);
			expect(timestamp).toBeLessThanOrEqual(result.captureEnd + CAPTURE_WINDOW_TOLERANCE_MS);
			expect(Math.abs(result.recordedDurationMs - result.measuredDurationMs)).toBeLessThan(100);
		} finally {
			fs.rmSync(root, { recursive: true, force: true });
		}
	}
};
