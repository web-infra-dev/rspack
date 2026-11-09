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

export default async function run(runtimeMode, status, acceptOutcome) {
	const context = await fs.realpath(
		await fs.mkdtemp(path.join(os.tmpdir(), "rspack-hmr-status-"))
	);
	const dependency = path.join(context, "dependency.js");
	const compiler = rspack({
		context,
		mode: "development",
		target: "node",
		entry: "./index.js",
		devtool: false,
		output: {
			path: path.join(context, "dist"),
			filename: "bundle.cjs",
			library: { type: "commonjs2" }
		},
		experiments: { runtimeMode },
		plugins: [new rspack.HotModuleReplacementPlugin()]
	});
	compiler.hooks.beforeRun.tap("AsyncHmrStatusPromises", () => {
		compiler.modifiedFiles = new Set([dependency]);
	});
	try {
		await fs.writeFile(path.join(context, "index.js"), `
require("./dependency");
export const statusError = new Error("${status} failed");
export const acceptError = new Error("accept failed");
let finishAccept;
let startAccept;
let failStatus;
const accepting = new Promise(resolve => { startAccept = resolve; });
const statusPromise = new Promise((_resolve, reject) => { failStatus = reject; });
module.hot.addStatusHandler(status => {
	if (status === ${JSON.stringify(status)}) return statusPromise;
});
module.hot.accept("./dependency", () => {
	startAccept();
	return new Promise((resolve, reject) => {
		finishAccept = () => ${acceptOutcome === "resolve" ? "resolve()" : "reject(acceptError)"};
	});
});
export const waitForAccept = () => accepting;
export const finish = () => finishAccept();
export const rejectStatus = () => failStatus(statusError);
export const update = () => module.hot.check(true);
`);
		await fs.writeFile(dependency, "module.exports = 0;");
		let stats = await runCompiler(compiler);
		assert(!stats.hasErrors(), stats.toString({ all: false, errors: true }));
		const require = createRequire(import.meta.url);
		const entry = require(path.join(context, "dist/bundle.cjs"));
		await fs.writeFile(dependency, "module.exports = 1;");
		compiler.inputFileSystem.purge();
		stats = await runCompiler(compiler);
		assert(!stats.hasErrors(), stats.toString({ all: false, errors: true }));
		const update = entry.update().catch(error => error);
		await entry.waitForAccept();
		if (acceptOutcome === "reject-before-status") {
			entry.finish();
			assert.equal(await update, entry.acceptError);
			entry.rejectStatus();
		} else {
			entry.rejectStatus();
			await setImmediate();
			entry.finish();
			assert.equal(await update, entry.statusError);
		}
		await setImmediate();
	} finally {
		await closeCompiler(compiler);
		await fs.rm(context, { recursive: true, force: true });
	}
}
