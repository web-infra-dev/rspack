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
		await fs.mkdtemp(path.join(os.tmpdir(), "rspack-hmr-recovery-"))
	);
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
			chunkLoadTimeout: 1000,
			library: { type: "commonjs2" }
		},
		experiments: { runtimeMode },
		plugins: [new rspack.HotModuleReplacementPlugin()]
	});
	let modifiedFiles;
	compiler.hooks.beforeRun.tap("AsyncHmrRecovery", () => {
		compiler.modifiedFiles = modifiedFiles;
	});
	const build = async (name, source, additional = {}) => {
		const changes = { [name]: source, ...additional };
		modifiedFiles = new Set();
		for (const [file, content] of Object.entries(changes)) {
			const fullPath = path.join(context, file);
			await fs.writeFile(fullPath, content);
			modifiedFiles.add(fullPath);
		}
		compiler.inputFileSystem.purge();
		const stats = await runCompiler(compiler);
		assert(!stats.hasErrors(), stats.toString({ all: false, errors: true }));
		return stats;
	};
	try {
		await fs.writeFile(
			path.join(context, "index.js"),
			scenario === "accept-error"
				? `
import { value } from "./dep";
export const events = [];
let first = true;
module.hot.accept("./dep", () => {
	if (first) { first = false; throw new Error("render failed"); }
	events.push("accepted");
}, error => events.push(error.message));
export const current = () => value;
export const update = () => module.hot.check(true);
export const status = () => module.hot.status();
`
				: `
require("./dep");
let lazy, started, release, nested;
exports.nestedAccepted = new Promise(resolve => { nested = resolve; });
exports.accepted = new Promise(resolve => { started = resolve; });
const gate = ${["chunk-version-late", "chunk-version-pending", "chunk-version-status-error"].includes(scenario) ? "new Promise(resolve => { release = resolve; })" : "Promise.resolve()"};
exports.release = () => release();
exports.events = [];
${
	scenario === "chunk-version-status-error"
		? `
let applyCount = 0;
module.hot.addStatusHandler(status => {
	if (status === "apply" && ++applyCount === 1) return new Promise((_, reject) => {
		exports.rejectStatus = () => reject(new Error("status failed"));
	});
});
`
		: ""
}
const load = () => import(/* webpackChunkName: "lazy" */ "./lazy");
module.hot.accept("./dep", () => {
	exports.events.push(require("./dep"));
	started();
	lazy = ${["chunk-version-pending", "chunk-version-status-error"].includes(scenario) ? "Promise.resolve()" : "gate"}.then(() => Promise.all([load(), load()])).then(([a, b]) => {
		if (a !== b) throw new Error("concurrent imports must share the module");
		return a.default;
	});
	if (require("./dep") === 2) {
		nested();
		${["chunk-version-pending", "chunk-version-status-error"].includes(scenario) ? "return lazy.then(() => gate);" : scenario === "chunk-version-error" ? 'return lazy.then(() => { throw new Error("nested accept failed"); });' : ""}
	}
	return lazy;
});
exports.update = () => ${scenario === "chunk-version-manual" ? "module.hot.check().then(() => module.hot.apply())" : "module.hot.check(true)"};
exports.current = () => lazy;
exports.status = () => module.hot.status();
`
		);
		await fs.writeFile(path.join(context, "lazy.js"), "export default 0;");
		const dependency = version =>
			scenario === "accept-error"
				? `export const value = await Promise.resolve(${version});`
				: `module.exports = ${version};`;
		await build("dep.js", dependency(0));
		const entry = await createRequire(import.meta.url)(
			path.join(context, "dist/bundle.cjs")
		);
		await build("dep.js", dependency(1));
		let latestUpdate;
		let update;
		if (scenario === "chunk-version-late") {
			update = entry.update();
			await entry.accepted;
			assert.equal(entry.status(), "apply");
		}
		if (scenario !== "accept-error") {
			latestUpdate = await build(
				"lazy.js",
				"export default 2;",
				[
					"chunk-version-same-dependency",
					"chunk-version-pending",
					"chunk-version-error",
					"chunk-version-status-error"
				].includes(scenario)
					? { "dep.js": dependency(2) }
					: {}
			);
		}
		if (scenario === "chunk-version-missing-manifest") {
			const manifest = latestUpdate.compilation
				.getAssets()
				.find(asset => asset.name.endsWith(".hot-update.json"))?.name;
			assert.ok(manifest);
			await fs.unlink(path.join(context, "dist", manifest));
			await assert.rejects(entry.update(), /no update leads/);
			await assert.rejects(entry.current(), /no update leads/);
			assert.equal(entry.status(), "fail");
			return;
		}
		if (scenario === "chunk-version-multiple")
			await build("lazy.js", "export default 3;");
		if (scenario === "chunk-version-error") {
			await assert.rejects(entry.update(), /nested accept failed/);
			assert.equal(entry.status(), "fail");
			return;
		}
		if (scenario === "chunk-version-status-error") {
			update = entry.update();
			await entry.nestedAccepted;
			assert.equal(await entry.current(), 2);
			const rejected = assert.rejects(update, /status failed/);
			entry.rejectStatus();
			await rejected;
			entry.release();
			await setImmediate();
			assert.equal(entry.status(), "apply");
			return;
		}
		if (scenario === "chunk-version-pending") {
			let finished = false;
			update = entry.update().then(ids => {
				finished = true;
				return ids;
			});
			await entry.nestedAccepted;
			assert.equal(await entry.current(), 2);
			assert.equal(finished, false);
			assert.equal(entry.status(), "apply");
			entry.release();
		}
		if (scenario === "chunk-version-late") entry.release();
		assert.ok((await (update || entry.update())).length);
		if (scenario === "chunk-version-same-dependency")
			assert.deepEqual(entry.events, [1, 2]);
		if (scenario === "accept-error") {
			assert.deepEqual(entry.events, ["render failed"]);
			assert.equal(entry.status(), "idle");
			await build("dep.js", dependency(2));
			assert.ok((await entry.update()).length);
			assert.deepEqual(entry.events, ["render failed", "accepted"]);
		}
		assert.equal(
			await entry.current(),
			scenario === "chunk-version-multiple" ? 3 : 2
		);
		assert.equal(entry.status(), "idle");
		await build("dep.js", dependency(3));
		assert.ok((await entry.update()).length);
		assert.equal(entry.status(), "idle");
	} finally {
		await closeCompiler(compiler);
		await fs.rm(context, { recursive: true, force: true });
	}
}
