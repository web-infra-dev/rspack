import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import { setTimeout as sleep } from "node:timers/promises";
import { rspack, lazyCompilationMiddleware } from "@rspack/core";

const require = createRequire(import.meta.url);

it.each([
	{ name: "root", entry: "app", filename: "[name].cjs" },
	{
		name: "nested runtime",
		entry: "app",
		filename: "static/js/[name].cjs",
		runtimeChunk: "single",
	},
	{ name: "nested entry", entry: "static/js/app", filename: "[name].cjs" },
	{
		name: "custom update paths",
		entry: "app",
		filename: "server/entry/[name].cjs",
		hotUpdateMainFilename: "updates/[runtime].[fullhash].manifest.json",
		hotUpdateChunkFilename: "updates/[id].[fullhash].update.cjs",
	},
	{
		name: "missing manifest",
		entry: "app",
		filename: "static/js/[name].cjs",
		removeManifest: true,
	},
])("loads Node hot updates from output.path: $name", async (shape) => {
	const dir = fs.mkdtempSync(path.join(os.tmpdir(), "rspack-hmr-root-"));
	const out = path.join(dir, "dist");
	fs.writeFileSync(
		path.join(dir, "entry.js"),
		`
  let dep = require('./dep.js');
  module.hot.accept('./dep.js', () => { dep = require('./dep.js'); });
  exports.read = () => dep.value;
  exports.update = () => module.hot.check(true);
  exports.lazy = () => import('./late.js').then(m => m.value);
 `,
	);
	fs.writeFileSync(path.join(dir, "dep.js"), "exports.value = 1;");
	fs.writeFileSync(path.join(dir, "late.js"), "exports.value = 'async';");
	const compiler = rspack({
		context: dir,
		mode: "development",
		target: "node",
		devtool: false,
		entry: { [shape.entry]: "./entry.js" },
		output: {
			path: out,
			filename: shape.filename,
			chunkFilename: "async/[name].cjs",
			library: { type: "commonjs2" },
			chunkFormat: "commonjs",
			chunkLoading: "require",
			...(shape.hotUpdateMainFilename
				? { hotUpdateMainFilename: shape.hotUpdateMainFilename }
				: {}),
			...(shape.hotUpdateChunkFilename
				? { hotUpdateChunkFilename: shape.hotUpdateChunkFilename }
				: {}),
		},
		optimization: {
			minimize: false,
			moduleIds: "named",
			chunkIds: "named",
			runtimeChunk: shape.runtimeChunk || false,
			splitChunks: false,
		},
		plugins: [new rspack.HotModuleReplacementPlugin()],
	});
	const builds = [];
	let failure;
	const watching = compiler.watch({ aggregateTimeout: 10 }, (error, stats) => {
		if (error || stats.hasErrors())
			failure = error || new Error(stats.toString());
		else builds.push(stats);
	});
	const nextBuild = async (count) => {
		const deadline = Date.now() + 10000;
		while (builds.length <= count && !failure && Date.now() < deadline)
			await sleep(10);
		if (failure) throw failure;
		expect(builds.length).toBeGreaterThan(count);
		return builds.at(-1);
	};
	try {
		await nextBuild(0);
		const entry = shape.filename.replace("[name]", shape.entry);
		const live = require(path.join(out, entry));
		expect(live.read()).toBe(1);
		expect(await live.lazy()).toBe("async");
		const count = builds.length;
		fs.writeFileSync(path.join(dir, "dep.js"), "exports.value = 2;");
		const stats = await nextBuild(count);
		const manifests = stats.compilation
			.getAssets()
			.filter(
				(a) =>
					a.name.endsWith(".hot-update.json") ||
					a.name.endsWith(".manifest.json"),
			);
		expect(manifests).toHaveLength(1);
		if (shape.removeManifest) fs.unlinkSync(path.join(out, manifests[0].name));
		const updated = await live.update();
		if (shape.removeManifest) {
			expect(updated).toBe(null);
			expect(live.read()).toBe(1);
		} else {
			expect(updated.length).toBeGreaterThan(0);
			expect(live.read()).toBe(2);
		}
	} finally {
		await new Promise((resolve, reject) =>
			compiler.close((error) => (error ? reject(error) : resolve())),
		);
		for (const key of Object.keys(require.cache))
			if (key.startsWith(dir + path.sep)) delete require.cache[key];
		fs.rmSync(dir, { recursive: true, force: true });
	}
});

it.each([
	{
		name: "development",
		mode: "development",
		lazy: true,
		hot: true,
		concatenate: false,
		enabled: true,
	},
	{
		name: "production",
		mode: "production",
		lazy: true,
		hot: true,
		concatenate: false,
		enabled: false,
	},
	{
		name: "concatenation",
		mode: "development",
		lazy: true,
		hot: true,
		concatenate: true,
		enabled: false,
	},
	{
		name: "no lazy plugin",
		mode: "development",
		lazy: false,
		hot: true,
		concatenate: false,
		enabled: false,
	},
	{
		name: "no HMR",
		mode: "development",
		lazy: true,
		hot: false,
		concatenate: false,
		enabled: false,
	},
])(
	"scopes the async block map to lazy development HMR: $name",
	async (policy) => {
		const dir = fs.mkdtempSync(path.join(os.tmpdir(), "rspack-map-😀-"));
		fs.writeFileSync(
			path.join(dir, "entry.js"),
			`
  exports.hasMap = typeof __webpack_require__.eb === 'function';
  exports.load = () => Promise.all([
   import(/* webpackChunkName: "😀-one" */ './one.js'),
   import(/* webpackChunkName: "😀-two", webpackFetchPriority: "high" */ './two.js'),
  ]).then(values => values.map(value => value.default));
 `,
		);
		fs.writeFileSync(path.join(dir, "one.js"), "export default 1;");
		fs.writeFileSync(path.join(dir, "two.js"), "export default 2;");
		const compiler = rspack({
			context: dir,
			mode: policy.mode,
			target: "node",
			devtool: false,
			entry: "./entry.js",
			output: {
				path: path.join(dir, "dist"),
				filename: "main.cjs",
				chunkFilename: "[name].cjs",
				library: { type: "commonjs2" },
			},
			// Keep these imports eager to isolate map policy and lossless Unicode decoding.
			lazyCompilation: policy.lazy
				? { entries: false, test: () => false }
				: false,
			optimization: {
				minimize: false,
				concatenateModules: policy.concatenate,
				chunkIds: "named",
			},
			plugins: policy.hot ? [new rspack.HotModuleReplacementPlugin()] : [],
		});
		if (policy.lazy) lazyCompilationMiddleware(compiler);
		try {
			const stats = await new Promise((resolve, reject) =>
				compiler.run((error, stats) =>
					error ? reject(error) : resolve(stats),
				),
			);
			expect(stats.hasErrors()).toBe(false);
			const live = require(path.join(dir, "dist/main.cjs"));
			expect(live.hasMap).toBe(policy.enabled);
			expect(await live.load()).toEqual([1, 2]);
		} finally {
			await new Promise((resolve, reject) =>
				compiler.close((error) => (error ? reject(error) : resolve())),
			);
			for (const key of Object.keys(require.cache))
				if (key.startsWith(dir + path.sep)) delete require.cache[key];
			fs.rmSync(dir, { recursive: true, force: true });
		}
	},
);
