const fs = require("node:fs");
const path = require("node:path");
const rspack = require("@rspack/core");
const { close, write } = require("./_module-cache");

const PLUGIN = "ModuleStateCacheTest";

function run(compiler, built) {
	built.length = 0;
	return new Promise((resolve, reject) => {
		compiler.run((error, stats) => {
			if (error) return reject(error);
			if (stats.hasErrors()) {
				return reject(new Error(stats.toString({ all: false, errors: true })));
			}
			resolve(stats);
		});
	});
}

function options(root, cache, built) {
	return {
		context: root,
		mode: "development",
		target: "node",
		entry: "./index.js",
		devtool: false,
		incremental: false,
		cache,
		experiments: {
			newCache: {
				module: true,
				loader: false,
				codeGeneration: false,
				devtool: false,
				minimize: false,
			},
		},
		resolve: { alias: { ignored: false } },
		externals: { external: "commonjs node:path" },
		module: {
			rules: [
				{
					test: /\.css$/,
					use: [
						rspack.CssExtractRspackPlugin.loader,
						require.resolve("css-loader"),
						"./metadata-loader.js",
					],
				},
			],
		},
		optimization: { minimize: false, concatenateModules: false },
		output: {
			path: path.join(root, "dist"),
			filename: "main.js",
			chunkFilename: "[name].[contenthash].js",
			library: { type: "commonjs2" },
		},
		plugins: [
			new rspack.CssExtractRspackPlugin({ filename: "main.css" }),
			{
				apply(compiler) {
					compiler.hooks.compilation.tap(PLUGIN, (compilation) => {
						compilation.hooks.buildModule.tap(PLUGIN, (module) => {
							built.push(module.identifier());
						});
					});
				},
			},
		],
	};
}

async function checkOutput(root, values, color) {
	const bundle = path.join(root, "dist/main.js");
	delete require.cache[bundle];
	const result = require(bundle);
	expect(result.values).toEqual(values);
	expect(await result.lazy()).toEqual(values);
	expect(result.external).toBe("file.js");
	expect(result.ignored).toEqual({});
	expect(fs.readFileSync(path.join(root, "dist/main.css"), "utf8")).toContain(
		`color: ${color}`,
	);
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
module.exports = {
	description:
		"should cache module states and invalidate context and CSS builds",
	options(context) {
		return { context: context.getSource(), entry: "./a" };
	},
	async build(context) {
		for (const persistent of [false, true]) {
			const root = context.getDist(persistent ? "persistent" : "memory");
			fs.rmSync(root, { recursive: true, force: true });
			write(
				root,
				"index.js",
				`
        require("./style.css");
        const sync = require.context("./context", false, /\\.js$/, "sync");
        const lazy = require.context("./context", false, /\\.js$/, "lazy-once");
        exports.values = sync.keys().sort().map(key => sync(key));
        exports.lazy = () => Promise.all(lazy.keys().sort().map(key => lazy(key)));
        exports.external = require("external").basename("/dir/file.js");
        exports.ignored = require("ignored");
      `,
			);
			write(root, "context/a.js", 'module.exports = "a";');
			write(root, "style.css", "body { color: red; }");
			write(
				root,
				"metadata-loader.js",
				`
        const fs = require("node:fs");
        const path = require("node:path");
        module.exports = function(source) {
          const config = path.join(this.rootContext, "css-deps.json");
          this.addDependency(config);
          const metadata = JSON.parse(fs.readFileSync(config, "utf8"));
          this.addDependency(path.join(this.rootContext, metadata.file));
          this.cacheable(metadata.cacheable);
          return source;
        };
      `,
			);
			const firstDependency = write(root, "first.txt", "first");
			const secondDependency = write(root, "second.txt", "second");
			const metadata = (file, cacheable = true) =>
				write(root, "css-deps.json", JSON.stringify({ file, cacheable }));
			metadata("first.txt");
			const built = [];
			const cache = persistent
				? {
						type: "persistent",
						storage: {
							type: "filesystem",
							directory: path.join(root, "cache"),
						},
					}
				: { type: "memory" };
			const create = () => rspack(options(root, cache, built));
			let compiler = create();
			try {
				await run(compiler, built);
				const initial = [...built];
				expect(initial.some((id) => id.startsWith("external "))).toBe(true);
				expect(initial.some((id) => id.startsWith("ignored|"))).toBe(true);
				expect(initial.some((id) => id.startsWith("css|"))).toBe(true);
				expect(
					initial.filter(
						(id) => id.includes("|sync") || id.includes("|lazy-once"),
					),
				).toHaveLength(2);
				await checkOutput(root, ["a"], "red");

				if (persistent) {
					await close(compiler);
					compiler = create();
				}
				await run(compiler, built);
				expect(built, JSON.stringify({ persistent, built })).toEqual([]);
				await checkOutput(root, ["a"], "red");

				metadata("second.txt");
				const stats = await run(compiler, built);
				expect(built.some((id) => id.startsWith("css|"))).toBe(false);
				expect([...stats.compilation.fileDependencies]).toContain(
					secondDependency,
				);
				expect([...stats.compilation.fileDependencies]).not.toContain(
					firstDependency,
				);

				metadata("second.txt", false);
				await run(compiler, built);
				expect(built.some((id) => id.startsWith("css|"))).toBe(true);
				metadata("second.txt");
				await run(compiler, built);
				expect(built.some((id) => id.startsWith("css|"))).toBe(true);
				await run(compiler, built);
				expect(built).toEqual([]);

				write(root, "context/b.js", 'module.exports = "b";');
				await run(compiler, built);
				expect(
					built.filter(
						(id) => id.includes("|sync") || id.includes("|lazy-once"),
					),
				).toHaveLength(2);
				expect(
					built.some(
						(id) => id.startsWith("external ") || id.startsWith("ignored|"),
					),
				).toBe(false);
				await checkOutput(root, ["a", "b"], "red");

				fs.unlinkSync(path.join(root, "context/a.js"));
				write(root, "style.css", "body { color: blue; }");
				await run(compiler, built);
				expect(built.some((id) => id.startsWith("css|"))).toBe(true);
				await checkOutput(root, ["b"], "blue");

				if (persistent) {
					await close(compiler);
					compiler = create();
				}
				await run(compiler, built);
				expect(built).toEqual([]);
				await checkOutput(root, ["b"], "blue");
			} finally {
				await close(compiler);
			}
		}
	},
};
