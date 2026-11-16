import fs from "node:fs";
import path from "node:path";
import { promisify } from "node:util";

const entry = "./loader-cache/index.js";

/** @type {import('@rspack/test-tools').TMultiCompilerCaseConfig[]} */
export default ["persistent", "memory"].map(type => ({
	description: `should reuse the ${type} loader cache across compilers`,
	options(context) {
		const cacheDirectory = context.getDist("cache");
		const counterFile = context.getDist("loader-runs.txt");
		fs.rmSync(cacheDirectory, { recursive: true, force: true });
		fs.rmSync(counterFile, { force: true });

		const createOptions = (name, dependencies) => ({
			name,
			dependencies,
			context: import.meta.dirname,
			mode: "development",
			entry,
			output: {
				path: context.getDist(name),
				filename: "bundle.js"
			},
			cache: type === "memory" ? { type } : {
				type: "persistent",
				storage: {
					type: "filesystem",
					directory: cacheDirectory
				}
			},
			experiments: {
				newCache: {
					codeGeneration: false,
					devtool: false,
					loader: true,
					minimize: false,
					module: false
				}
			},
			module: {
				rules: [
					{
						test: /index\.js$/,
						use: [
							{
								loader: path.join(
									import.meta.dirname,
									"loader-cache/count-loader.mjs"
								),
								options: { counterFile },
								cache: true
							}
						]
					}
				]
			}
		});

		return [createOptions("a"), createOptions("b", ["a"])];
	},
	async build(context, compiler) {
		if (type === "memory") {
			const [parent] = compiler.compilers;
			let sharedCacheShutdowns = 0;
			parent.cache.hooks.shutdown.tap("CloseChild", () => {
				sharedCacheShutdowns++;
			});
			parent.hooks.done.tapPromise("CloseChild", async ({ compilation }) => {
				for (const name of ["first-child", "disabled-child", "second-child"]) {
					const child = compilation.createChildCompiler(
						name,
						{ filename: `${name}.js` },
						[new parent.rspack.EntryPlugin(parent.context, entry, { name })]
					);
					child.options.experiments = {
						...child.options.experiments,
						newCache: {
							...child.options.experiments.newCache,
							loader: name !== "disabled-child"
						}
					};
					try {
						await promisify(child.runAsChild.bind(child))();
						const { cacheInfo } = child._lastCompilation.getStats().toJson({
							all: false,
							cacheInfo: true
						});
						expect(cacheInfo?.[`${child.compilerPath}|loader`]).toEqual(
							name === "disabled-child"
								? undefined
								: { hits: 1, misses: 0, reused: 1, stores: 0 }
						);
					} finally {
						await promisify(child.close.bind(child))();
					}
				}
				expect(sharedCacheShutdowns).toBe(0);
			});
		}
		try {
			const stats = await promisify(compiler.run.bind(compiler))();
			expect(stats.hasErrors()).toBe(false);
			const cacheInfo = stats.stats.map((stats, index) => {
				const { cacheInfo } = stats.toJson({ all: false, cacheInfo: true });
				return cacheInfo[`${compiler.compilers[index].compilerPath}|loader`];
			});
			expect(cacheInfo).toEqual([
				{ hits: 0, misses: 1, reused: 0, stores: 1 },
				{ hits: 1, misses: 0, reused: 1, stores: 0 }
			]);
			expect(fs.readFileSync(context.getDist("loader-runs.txt"), "utf-8")).toBe(
				type === "memory" ? "xx" : "x"
			);
		} finally {
			await promisify(compiler.close.bind(compiler))();
		}
	}
}));
