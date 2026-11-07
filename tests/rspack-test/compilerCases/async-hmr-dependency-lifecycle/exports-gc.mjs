import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import { rspack } from "@rspack/core";
import {
	closeCompiler,
	createGCTracker,
	runCompiler
} from "@rspack/test-tools/helper/lifecycle";

export default async function run(runtimeMode) {
	const context = await fs.realpath(await fs.mkdtemp(path.join(os.tmpdir(), "rspack-async-hmr-gc-")));
	const dependency = path.join(context, "dependency.js");
	const tracker = createGCTracker();
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
	compiler.hooks.beforeRun.tap("AsyncHmrDependencyLifecycle", () => {
		compiler.modifiedFiles = new Set([dependency]);
	});
	await fs.writeFile(path.join(context, "index.js"), `
import { payload } from "./dependency";
module.hot.accept("./dependency");
export const current = () => payload;
export const update = () => module.hot.check(true);
`);
	const require = createRequire(import.meta.url);
	try {
		let entry;
		for (let version = 0; version < 4; version++) {
			await fs.writeFile(dependency, `
await Promise.resolve();
export const payload = { version: ${version}, data: new Uint8Array(1024 * 1024) };
`);
			compiler.inputFileSystem.purge();
			const stats = await runCompiler(compiler);
			assert(!stats.hasErrors(), stats.toString({ all: false, errors: true }));
			if (version === 0) {
				entry = await require(path.join(context, "dist/bundle.cjs"));
			} else {
				await entry.update();
			}
			assert.equal(entry.current().version, version);
			if (version < 3) tracker.track(entry.current(), `exports ${version}`);
		}
		for (let version = 0; version < 3; version++) {
			await tracker.waitForCollection(`exports ${version}`);
		}
		assert.equal(entry.current().version, 3);
	} finally {
		await closeCompiler(compiler);
		await fs.rm(context, { recursive: true, force: true });
	}
}
