import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import vm from "node:vm";
import { rspack } from "@rspack/core";

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default ["webpack", "rspack"].map(runtimeMode => ({
	description: `updates chunk group dependencies during apply (${runtimeMode})`,
	options(context) {
		return {
			context: context.getDist("input"),
			entry: { main: "./index.js", separate: "./separate.js" },
			mode: "development",
			devtool: false,
			target: "web",
			experiments: { runtimeMode },
			output: {
				path: context.getDist("output"),
				filename: "[name].js",
				chunkFilename: "[name].js",
				publicPath: "/"
			},
			optimization: {
				moduleIds: "named",
				chunkIds: "named",
				splitChunks: {
					cacheGroups: {
						default: false,
						defaultVendors: false,
						first: { test: /[\\/]first\.js$/, name: "first", enforce: true },
						second: { test: /[\\/]second\.js$/, name: "second", enforce: true },
						stable: { test: /[\\/]stable\.js$/, name: "stable", enforce: true }
					}
				}
			},
			plugins: [new rspack.HotModuleReplacementPlugin()]
		};
	},
	async build(context, compiler) {
		const input = context.getDist("input");
		fs.mkdirSync(input, { recursive: true });
		const write = (file, source) => fs.writeFileSync(path.join(input, file), source);
		write("index.js", `
			globalThis.runtime = {
				get hmrS_chunkGroups() { return __webpack_require__.hmrS_chunkGroups; },
				get f() { return __webpack_require__.f; },
				e: (id, priority) => __webpack_require__.e(id, priority)
			};
			globalThis.hot = module.hot;
			globalThis.load = () => import(/* webpackChunkName: 'page' */ './page');
			globalThis.other = () => import(/* webpackChunkName: 'other', webpackFetchPriority: 'high' */ './other');
			module.hot.accept();
		`);
		write("other.js", "module.exports = require('./stable');");
		write("separate.js", "import(/* webpackChunkName: 'private' */ './private');");
		write("private.js", "module.exports = require('./stable');");
		for (const file of ["first", "second", "stable"]) write(`${file}.js`, "module.exports = 1;");
		const builds = [];
		for (const source of ["require('./first')", "require('./second')", "3"]) {
			write("page.js", `module.exports = ${source};`);
			const stats = await new Promise((resolve, reject) => compiler.run(
				(error, stats) => error ? reject(error) : resolve(stats),
				{ modifiedFiles: new Set([path.join(input, "page.js")]) }
			));
			assert.equal(stats.hasErrors(), false, stats.toString());
			builds.push(Object.fromEntries(stats.compilation.getAssets().map(asset => [asset.name, asset.source.source().toString()])));
		}
		let assets = builds[0];
		const sandbox = vm.createContext({ console, Promise, setTimeout, clearTimeout });
		sandbox.self = sandbox;
		sandbox.document = {
			getElementsByTagName: () => [],
			createElement: () => ({ setAttribute() {} }),
			head: {
				appendChild(script) {
					vm.runInContext(assets[path.basename(script.src.split("?")[0])], sandbox);
					queueMicrotask(() => script.onload({ type: "load", target: script }));
				}
			}
		};
		sandbox.fetch = async url => ({ ok: true, json: async () => JSON.parse(assets[path.basename(url)]) });
		vm.runInContext(assets["main.js"], sandbox);
		const readMap = () => JSON.parse(JSON.stringify(sandbox.runtime.hmrS_chunkGroups));
		assert.deepEqual(readMap(), { other: ["stable"], page: ["first"] });
		for (let step = 1; step < builds.length; step++) {
			const before = readMap();
			assets = builds[step];
			await sandbox.hot.check(false);
			assert.deepEqual(readMap(), before);
			const update = Object.entries(assets).filter(([name]) => name.endsWith(".hot-update.js")).map(([, source]) => source).join("\n");
			assert(!update.includes('"other":["stable"]'), "unchanged groups must not be sent again");
			await sandbox.hot.apply();
			assert.deepEqual(readMap(), step === 1 ? { other: ["stable"], page: ["second"] } : { other: ["stable"] });
		}
		// Exercise e's existing handler contract, including fetch priority. The
		// group handler must not recursively expand another representative chunk.
		const { runtime } = sandbox;
		const saved = { ...runtime.f };
		const requested = [];
		try {
			for (const key of Object.keys(runtime.f)) delete runtime.f[key];
			runtime.f.group = saved.group;
			runtime.f.record = (id, promises, priority) => requested.push([id, priority]);
			await runtime.e("other", "high");
			assert.deepEqual(requested, [["stable", "high"], ["other", "high"]]);
		} finally {
			for (const key of Object.keys(runtime.f)) delete runtime.f[key];
			Object.assign(runtime.f, saved);
		}
		assert.equal((await sandbox.load()).default, 3);
	}
}));
