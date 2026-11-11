import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import { setImmediate } from "node:timers/promises";
import { rspack } from "@rspack/core";
import {
	closeCompiler,
	runCompiler
} from "@rspack/test-tools/helper/lifecycle";

export default async function run(runtimeMode, chunkLoading, scenario) {
	const context = await fs.realpath(
		await fs.mkdtemp(path.join(os.tmpdir(), "rspack-hmr-overlap-"))
	);
	const parentFailure = scenario === "parent-status-error";
	const compiler = rspack({
		context,
		mode: "development",
		target: "node",
		entry: "./index.js",
		devtool: false,
		output: {
			path: path.join(context, "dist"),
			filename: "bundle.cjs",
			chunkFilename: "[name].cjs",
			chunkLoading,
			chunkLoadTimeout: 3000,
			library: { type: "commonjs2" }
		},
		experiments: { runtimeMode },
		plugins: [new rspack.HotModuleReplacementPlugin()]
	});
	let modifiedFiles;
	compiler.hooks.beforeRun.tap("AsyncHmrOverlap", () => {
		compiler.modifiedFiles = modifiedFiles;
	});
	const build = async changes => {
		modifiedFiles = new Set();
		for (const [name, source] of Object.entries(changes)) {
			const file = path.join(context, name);
			await fs.writeFile(file, source);
			modifiedFiles.add(file);
		}
		compiler.inputFileSystem.purge();
		const stats = await runCompiler(compiler);
		assert(!stats.hasErrors(), stats.toString({ all: false, errors: true }));
	};
	let releaseOld;
	globalThis.__hmrOverlapGate = new Promise((resolve, reject) => {
		releaseOld =
			scenario === "stale-rejection"
				? () => reject(new Error("obsolete dependency"))
				: resolve;
	});
	try {
		await build({
			"index.js": `
import value from "./dep";
require("./trigger");
export const hot = module.hot;
export const current = () => value;
export const events = [];
let nested, prepared, release, rejectStatus, lazy;
export const nestedAccepted = new Promise(resolve => { nested = resolve; });
export const nestedPreparing = new Promise(resolve => { prepared = resolve; });
export const resume = () => release();
export const fail = () => rejectStatus(new Error("parent status failed"));
export const loaded = () => lazy;
let prepares = 0, applies = 0;
module.hot.addStatusHandler(status => {
	if (!${parentFailure}) return;
	if (status === "apply" && ++applies === 1) return new Promise((_, reject) => { rejectStatus = reject; });
	if (status === "prepare" && ++prepares === 2) {
		prepared();
		return new Promise(resolve => { release = resolve; });
	}
});
module.hot.accept("./dep", () => {
	events.push(value);
	if (value === 2) nested();
});
module.hot.accept("./trigger", () => {
	lazy = import(/* webpackChunkName: "lazy" */ "./lazy").then(m => m.default);
	return lazy;
});
`,
			"dep.js": "export default await Promise.resolve(0);",
			"trigger.js": "module.exports = 0;",
			"lazy.js": "export default 0;"
		});
		const entry = await createRequire(import.meta.url)(
			path.join(context, "dist/bundle.cjs")
		);
		await build({
			"dep.js": parentFailure
				? "export default await Promise.resolve(1);"
				: "await globalThis.__hmrOverlapGate; export default 1;",
			"trigger.js": "module.exports = 1;"
		});
		await build({
			"dep.js": "export default await Promise.resolve(2);",
			"lazy.js": "export default 2;"
		});
		const outcome = entry.hot.check(true).then(
			ids => ({ ids }),
			error => ({ error })
		);
		if (parentFailure) {
			await entry.nestedPreparing;
			entry.fail();
			assert.match((await outcome).error?.message || "", /parent status failed/);
			entry.resume();
			await entry.loaded();
			await entry.nestedAccepted;
			await setImmediate();
			assert.notEqual(
				entry.hot.status(),
				"idle",
				"a child must not recover a failed parent"
			);
		} else {
			await entry.nestedAccepted;
			assert.equal(entry.current(), 2);
			releaseOld();
			const result = await outcome;
			assert.ifError(result.error);
			assert.equal(
				entry.current(),
				2,
				"obsolete async evaluation must not overwrite the latest binding"
			);
			assert.deepEqual(entry.events, [2, 2]);
			assert.equal(entry.hot.status(), "idle");
		}
	} finally {
		releaseOld();
		delete globalThis.__hmrOverlapGate;
		await closeCompiler(compiler);
		await fs.rm(context, { recursive: true, force: true });
	}
}
