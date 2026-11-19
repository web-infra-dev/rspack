import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import { setImmediate } from "node:timers/promises";
import { rspack } from "@rspack/core";
import { closeCompiler, runCompiler } from "@rspack/test-tools/helper/lifecycle";

export default async function run(runtimeMode, chunkLoading, scenario, outcome) {
	const context = await fs.realpath(
		await fs.mkdtemp(path.join(os.tmpdir(), "rspack-hmr-acceptor-"))
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
			library: { type: "commonjs2" }
		},
		experiments: { runtimeMode },
		plugins: [new rspack.HotModuleReplacementPlugin()]
	});
	let modifiedFiles;
	compiler.hooks.beforeRun.tap("AcceptorLifetime", () => {
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
	let resolveGate, rejectGate, resolveUpdated;
	const gate = new Promise((resolve, reject) => {
		resolveGate = resolve;
		rejectGate = reject;
	});
	const updated = new Promise(resolve => { resolveUpdated = resolve; });
	const state = {
		gate,
		events: [],
		disposed: 0,
		release: () => {
			if (outcome === "reject") rejectGate(new Error("obsolete dependency"));
			else if (outcome === "resolve") resolveGate();
			resolveUpdated();
		}
	};
	globalThis.__hmrAcceptor = state;
	try {
		await build({
			"index.js": `
const state = globalThis.__hmrAcceptor;
module.exports = { hot: module.hot, ready: require("./container") };
require("./trigger");
module.hot.accept("./container", () => { require("./container"); });
module.hot.accept("./trigger", () => import(/* webpackChunkName: "lazy" */ "./lazy").then(m => {
	state.events.push(m.default);
	state.release();
}));
${scenario === "acceptor-replaced" ? `
require("./dep");
module.hot.accept("./dep", () => { require("./dep"); });` : ""}
`,
			"container.js": `
module.exports = require("./parent");
module.hot.accept("./parent", () => { module.exports = require("./parent"); });
`,
			"parent.js": `
import value from "./dep";
globalThis.__hmrAcceptor.oldHot = module.hot;
module.hot.dispose(() => { globalThis.__hmrAcceptor.disposed++; });
module.hot.accept("./dep", () => { globalThis.__hmrAcceptor.events.push("obsolete:" + value); });
export default value;
`,
			"dep.js": "export default await Promise.resolve(0);",
			"trigger.js": "module.exports = 0;",
			"lazy.js": 'export default "lazy0";'
		});
		const entry = createRequire(import.meta.url)(path.join(context, "dist/bundle.cjs"));
		await entry.ready;
		await build({
			"dep.js": "await globalThis.__hmrAcceptor.gate; export default 1;",
			"trigger.js": "module.exports = 1;"
		});
		await build({
			[scenario === "acceptor-removed" ? "container.js" : "parent.js"]:
				'globalThis.__hmrAcceptor.events.push("current"); module.exports = 2;',
			"lazy.js": 'export default "lazy2";'
		});
		let finished = false;
		const applying = entry.hot.check(true).then(
			ids => { finished = true; return { ids }; },
			error => { finished = true; return { error }; }
		);
		if (outcome === "pending") {
			await updated;
			await setImmediate();
			assert(finished, "a disposed acceptor must not wait for its old dependency");
		}
		assert.ifError((await applying).error);
		assert.equal(state.oldHot.active, false);
		assert.equal(state.disposed, 1);
		assert.equal(state.oldHot._disposeHandlers.length, 1);
		assert.deepEqual(state.events, ["current", "lazy2"]);
		assert.equal(entry.hot.status(), "idle");
	} finally {
		resolveGate();
		delete globalThis.__hmrAcceptor;
		await closeCompiler(compiler);
		await fs.rm(context, { recursive: true, force: true });
	}
}
