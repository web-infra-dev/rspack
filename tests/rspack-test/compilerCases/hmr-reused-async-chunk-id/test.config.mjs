import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import vm from "node:vm";
import { rspack } from "@rspack/core";

// A loaded async chunk whose only module is replaced keeps the same chunk id, but loses every exclusive
// module: the new chunk contains `b.js` instead of `a.js`. HMR must still send a regular incremental update
// for the current chunk; an id-reuse full update is never loaded for an already installed chunk.
// https://github.com/web-infra-dev/rspack/issues/16875

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
export default {
	description:
		"applies a reused async chunk id update to the modules of the current chunk",
	options(context) {
		return {
			context: context.getDist("input"),
			entry: { main: "./main.js" },
			mode: "development",
			devtool: false,
			target: "web",
			output: {
				path: context.getDist("output"),
				filename: "[name].js",
				chunkFilename: "[name].js",
				publicPath: "/"
			},
			optimization: {
				moduleIds: "named",
				chunkIds: "named",
				usedExports: false,
				concatenateModules: false,
				removeAvailableModules: false
			},
			plugins: [new rspack.HotModuleReplacementPlugin()]
		};
	},
	async build(context, compiler) {
		const input = context.getDist("input");
		fs.mkdirSync(input, { recursive: true });
		const write = (name, source) =>
			fs.writeFileSync(path.join(input, name), source);

		write("a.js", "module.exports = 'A'; module.hot.accept();");
		write("b.js", "module.exports = 'B'; module.hot.accept();");
		write(
			"main.js",
			"globalThis.req = __webpack_require__;\nglobalThis.hot = module.hot;\nglobalThis.load = () => import(/* webpackChunkName: 'page' */ './a');\nmodule.hot.accept();"
		);

		const builds = [];
		for (let step = 0; step < 2; step++) {
			if (step === 1) {
				write(
					"main.js",
					"globalThis.req = __webpack_require__;\nglobalThis.hot = module.hot;\nglobalThis.load = () => import(/* webpackChunkName: 'page' */ './b');\nmodule.hot.accept();"
				);
			}
			const stats = await new Promise((resolve, reject) =>
				compiler.run(
					(error, stats) => (error ? reject(error) : resolve(stats)),
					{
						modifiedFiles: new Set(
							["main.js", "b.js"].map(name => path.join(input, name))
						)
					}
				)
			);
			assert.equal(stats.hasErrors(), false, stats.toString());
			builds.push(
				Object.fromEntries(
					stats.compilation
						.getAssets()
						.map(asset => [asset.name, asset.source.source().toString()])
				)
			);
		}

		const [initialAssets, updatedAssets] = builds;
		const sandbox = vm.createContext({
			self: {},
			console,
			Promise,
			setTimeout,
			clearTimeout
		});
		vm.runInContext(initialAssets["main.js"], sandbox);
		const { req, hot } = sandbox;
		const requests = [];
		let assets = initialAssets;
		req.l = (url, done) => {
			const name = path.basename(url);
			requests.push(name);
			vm.runInContext(assets[name], sandbox);
			done({ type: "load" });
		};

		const first = await sandbox.load();
		assert.equal(first.default, "A");
		assert(requests.includes("page.js"), requests.join(", "));

		assets = updatedAssets;
		const manifestName = Object.keys(assets).find(name =>
			name.endsWith(".hot-update.json")
		);
		assert(manifestName, "missing hot update manifest");
		const manifest = JSON.parse(assets[manifestName]);
		assert(
			manifest.c.includes("page"),
			`reused async chunk is missing from the incremental manifest: ${JSON.stringify(manifest)}`
		);
		req.hmrM = async () => manifest;

		await hot.check(true);
		assert.equal(hot.status(), "idle");

		const second = await sandbox.load();
		assert.equal(second.default, "B");
	}
};
