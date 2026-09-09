const fs = require("node:fs");
const path = require("node:path");
const { createRequire } = require("node:module");

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
module.exports = [
	{ name: "memory", cache: true },
	{ name: "persistent-reopen", cache: "persistent", reopen: true },
	{ name: "legacy-persistent-reopen", cache: "persistent", reopen: true, legacy: true },
	{ name: "disabled", cache: false }
].map(({ name, cache, reopen, legacy }) => {
	let root;
	let timestamp;
	let selected = "a";
	let built;

	function write(file, source) {
		const filename = path.join(root, file);
		fs.writeFileSync(filename, source);
		fs.utimesSync(filename, timestamp, timestamp);
	}

	function writeEntry() {
		write("index.js", `
			import { value } from "./${selected}.js";
			export const sync = value;
			export const async = () => import("./${selected}-async.js").then(m => m.default);
		`);
	}

	return {
		description: `should retain dependency and block objects across graph replacement with ${name}`,
		options(context) {
			root = context.getDist(name);
			fs.rmSync(root, { recursive: true, force: true });
			fs.mkdirSync(root, { recursive: true });
			timestamp = new Date(Date.now() - 20000);
			writeEntry();
			for (const value of ["a", "b"]) {
				write(`${value}.js`, `export const value = "${value}";`);
				write(`${value}-async.js`, `export default "${value}";`);
			}
			return {
				context: root,
				entry: "./index.js",
				target: "node",
				mode: "production",
				devtool: false,
				incremental: false,
				output: {
					path: path.join(root, "dist"),
					filename: "main.js",
					chunkFilename: "[id].[contenthash].js",
					library: { type: "commonjs2" }
				},
				optimization: { concatenateModules: true, inlineExports: false, minimize: false },
				cache: cache === "persistent" ? {
					type: "persistent",
					storage: { type: "filesystem", location: path.join(root, "cache") }
				} : cache,
				experiments: legacy ? {} : {
					newCache: { module: true, loader: false, codeGeneration: false, devtool: false, minimize: false }
				},
				plugins: [{
					apply(compiler) {
						compiler.hooks.compilation.tap("ModuleOwnedDependenciesTest", compilation => {
							compilation.hooks.buildModule.tap("ModuleOwnedDependenciesTest", module => {
								if (module.resource === path.join(root, "index.js")) built++;
							});
							compilation.hooks.finishModules.tap("ModuleOwnedDependenciesTest", modules => {
								const entry = [...modules].find(module => module.resource === path.join(root, "index.js"));
								const dependency = entry.dependencies.find(dep => dep.request === `./${selected}.js`);
								expect(dependency).toBeDefined();
								expect(compilation.moduleGraph.getModule(dependency).resource).toBe(path.join(root, `${selected}.js`));
								expect(entry.blocks).toHaveLength(1);
								const asyncDependency = entry.blocks[0].dependencies[0];
								expect(asyncDependency.request).toBe(`./${selected}-async.js`);
								expect(compilation.moduleGraph.getModule(asyncDependency).resource).toBe(path.join(root, `${selected}-async.js`));
							});
						});
					}
				}]
			};
		},
		compiler(_, compiler) {
			compiler.outputFileSystem = fs;
		},
		async build(context) {
			const manager = context.getCompiler();
			for (let iteration = 0; iteration < 5; iteration++) {
				built = 0;
				timestamp = new Date(timestamp.getTime() + 1000);
				if (iteration === 2 || iteration === 4) {
					selected = iteration === 2 ? "b" : "a";
					writeEntry();
				}
				if (iteration === 3) write("b-async.js", 'export default "updated";');
				const stats = await manager.build();
				expect(stats.toJson({ all: false, errors: true }).errors).toEqual([]);
				if (!legacy) {
					// Consecutive run() calls use rebuild(), which bypasses the module cache.
					const cacheHit = reopen && iteration % 2 === 1;
					expect(built).toBe(cacheHit ? 0 : 1);
				}
				const filename = path.join(root, "dist/main.js");
				const source = fs.readFileSync(filename, "utf8");
				expect(source).toContain(`CONCATENATED MODULE: ./${selected}.js`);
				const output = { exports: {} };
				new Function("require", "module", "exports", source)(
					createRequire(filename), output, output.exports
				);
				expect(output.exports.sync).toBe(selected);
				expect(await output.exports.async()).toBe(iteration === 3 ? "updated" : selected);
				if (reopen && iteration < 4) {
					await manager.close();
					manager.createCompiler().outputFileSystem = fs;
				}
			}
		}
	};
});
