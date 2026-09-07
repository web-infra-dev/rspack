const fs = require("node:fs");
const path = require("node:path");
const rspack = require("@rspack/core");
const { close, write } = require("./_module-cache");

async function checkCache(root, persistent, configure, expectedPrefixes) {
	fs.mkdirSync(root, { recursive: true });
	write(root, "value.js", 'module.exports = "value";');
	write(root, "shared.js", 'module.exports = "shared";');
	const built = [];
	const create = () =>
		rspack({
			context: root,
			mode: "development",
			target: "node",
			devtool: false,
			incremental: false,
			entry: "./index.js",
			cache: persistent
				? {
						type: "persistent",
						storage: {
							type: "filesystem",
							directory: path.join(root, "cache"),
						},
					}
				: { type: "memory" },
			experiments: {
				newCache: {
					module: true,
					loader: false,
					codeGeneration: false,
					devtool: false,
					minimize: false,
				},
			},
			optimization: { concatenateModules: false, minimize: false },
			output: {
				path: path.join(root, "dist"),
				filename: "main.js",
				library: { type: "commonjs2" },
			},
			plugins: [
				...configure(root),
				{
					apply(compiler) {
						compiler.hooks.compilation.tap(
							"SyntheticModuleCacheTest",
							(compilation) => {
								compilation.hooks.buildModule.tap(
									"SyntheticModuleCacheTest",
									(module) => {
										built.push(module.identifier());
									},
								);
							},
						);
					},
				},
			],
		});
	let compiler = create();
	const run = () =>
		new Promise((resolve, reject) => {
			built.length = 0;
			compiler.run((error, stats) => {
				if (error) return reject(error);
				if (stats.hasErrors()) {
					return reject(
						new Error(stats.toString({ all: false, errors: true })),
					);
				}
				resolve();
			});
		});
	try {
		await run();
		for (const prefix of expectedPrefixes) {
			expect(
				built.filter((id) => id.startsWith(prefix)).length,
			).toBeGreaterThan(0);
		}
		const first = fs.readFileSync(path.join(root, "dist/main.js"), "utf8");
		if (persistent) {
			await close(compiler);
			compiler = create();
		}
		await run();
		expect(built, JSON.stringify({ root, persistent, built })).toEqual([]);
		expect(fs.readFileSync(path.join(root, "dist/main.js"), "utf8")).toBe(
			first,
		);
	} finally {
		await close(compiler);
	}
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
module.exports = {
	description: "should restore DLL and federation module states",
	options(context) {
		return { context: context.getSource(), entry: "./a" };
	},
	async build(context) {
		for (const persistent of [false, true]) {
			const root = context.getDist(persistent ? "persistent" : "memory");
			fs.rmSync(root, { recursive: true, force: true });
			await checkCache(
				path.join(root, "federation"),
				persistent,
				(root) => {
					if (!fs.existsSync(path.join(root, "index.js")))
						write(
							root,
							"index.js",
							`
          exports.remote = () => import("remote/value");
          exports.shared = () => import("shared-value");
        `,
						);
					return [
						new rspack.container.ModuleFederationPluginV1({
							name: "container",
							filename: "container.js",
							library: { type: "commonjs2" },
							exposes: { "./value": "./value.js" },
							remotes: { remote: ["commonjs remote-a", "commonjs remote-b"] },
							shared: {
								"shared-value": {
									import: "./shared.js",
									requiredVersion: false,
								},
							},
						}),
					];
				},
				[
					"container entry",
					"remote ",
					"fallback ",
					"consume shared module",
					"provide shared module",
				],
			);

			await checkCache(
				path.join(root, "dll"),
				persistent,
				(root) => {
					if (!fs.existsSync(path.join(root, "index.js")))
						write(root, "index.js", 'module.exports = require("./value");');
					return [
						new rspack.DllPlugin({ path: path.join(root, "manifest.json") }),
					];
				},
				["dll "],
			);

			await checkCache(
				path.join(root, "delegated"),
				persistent,
				(root) => {
					if (!fs.existsSync(path.join(root, "index.js")))
						write(root, "index.js", 'module.exports = require("dll/value");');
					return [
						new rspack.DllReferencePlugin({
							name: 'function() { return "delegated"; }',
							scope: "dll",
							content: { "./value": { id: 1, buildMeta: {}, exports: true } },
						}),
					];
				},
				["delegated "],
			);
		}
	},
};
