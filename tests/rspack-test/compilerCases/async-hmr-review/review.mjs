import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import { rspack } from "@rspack/core";
import {
	closeCompiler,
	runCompiler
} from "@rspack/test-tools/helper/lifecycle";

export default async function run(runtimeMode, chunkLoading, scenario) {
	const context = await fs.realpath(
		await fs.mkdtemp(path.join(os.tmpdir(), "rspack-hmr-review-"))
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
			chunkLoadTimeout: 3000,
			environment: { arrowFunction: !scenario.endsWith("noarrow") },
			library: { type: "commonjs2" }
		},
		optimization: { moduleIds: "named" },
		experiments: { runtimeMode },
		plugins: [new rspack.HotModuleReplacementPlugin()]
	});
	let modifiedFiles;
	compiler.hooks.beforeRun.tap("AsyncHmrReview", () => {
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
	try {
		if (scenario.startsWith("await-") || scenario.startsWith("yield-")) {
			const dependency = value =>
				`export default ${scenario.endsWith("-sync") ? value : `await Promise.resolve(${value})`};`;
			const callback = `(evaluations++, () => { values.push([value, this.label]); })`;
			const hot =
				runtimeMode === "webpack" ? "module.hot" : "import.meta.webpackHot";
			await build({
				"index.js": `
import value from "./dep";
export const hot = ${hot};
export const values = [];
export let evaluations = 0;
${
	scenario.startsWith("await-")
		? `
export const ready = (async function() {
	${hot}.accept("./dep", (await Promise.resolve(), ${callback}));
}).call({ label: "registered" });`
		: `
function* setup() {
	${hot}.accept("./dep", yield ${callback});
}
const iterator = setup.call({ label: "registered" });
iterator.next(iterator.next().value);
export const ready = Promise.resolve();`
}
`,
				"dep.js": dependency(0)
			});
			const entry = await createRequire(import.meta.url)(
				path.join(context, "dist/bundle.cjs")
			);
			await entry.ready;
			assert.equal(
				entry.evaluations,
				1,
				"evaluate the callback during registration"
			);
			for (const value of [1, 2]) {
				await build({ "dep.js": dependency(value) });
				await entry.hot.check(true);
				assert.equal(
					entry.evaluations,
					1,
					"do not reevaluate the callback on updates"
				);
			}
			assert.deepEqual(entry.values, [
				[1, "registered"],
				[2, "registered"]
			]);
			assert.equal(entry.hot.status(), "idle");
			return;
		}
		const removedSelf = scenario.startsWith("removed-self");
		await build({
			"index.js": `
require("./trigger");
require("./parent");
require("./peer");
require("./common");
export const hot = module.hot;
export const events = [];
module.hot.accept("./parent", () => { events.push(require("./parent")); });
module.hot.accept("./peer", () => { events.push(require("./peer")); });
module.hot.accept("./common", () => { require("./common"); });
module.hot.accept("./trigger", () => import(/* webpackChunkName: "lazy" */ "./lazy").then(m => { events.push(m.default); }));
`,
			"parent.js": 'module.exports = require("./self");',
			"self.js": "module.hot.accept(); module.exports = 0;",
			"asset.png": "asset",
			"peer.js": 'module.exports = "peer0";',
			"common.js": "module.exports = 0;",
			"trigger.js": "module.exports = 0;",
			"lazy.js": 'export default "lazy0";'
		});
		const entry = await createRequire(import.meta.url)(
			path.join(context, "dist/bundle.cjs")
		);
		await build({
			"trigger.js": "module.exports = 1;",
			...(removedSelf
				? {
						"self.js": scenario === "removed-self-runtime"
							? 'new URL("./asset.png", import.meta.url); module.hot.accept(); module.exports = 1;'
							: "module.hot.accept(); module.exports = 1;"
					}
				: { "common.js": "module.exports = 1;" })
		});
		await build({
			"lazy.js": 'export default "lazy2";',
			...(removedSelf
				? { "parent.js": 'module.exports = "removed";' }
				: {
						"peer.js": 'module.exports = "peer2";',
						"common.js": "module.exports = 2;"
					})
		});
		const ids = await entry.hot.check(true);
		assert.equal(entry.hot.status(), "idle");
		assert.deepEqual(entry.events, [removedSelf ? "removed" : "peer2", "lazy2"]);
		if (!removedSelf) {
			for (const id of ["./trigger.js", "./common.js", "./peer.js"]) {
				assert(ids.includes(id), `updated module ${id} is missing from ${ids}`);
			}
			assert.equal(
				new Set(ids).size,
				ids.length,
				"deduplicate IDs across updates"
			);
		}
	} finally {
		await closeCompiler(compiler);
		await fs.rm(context, { recursive: true, force: true });
	}
}
