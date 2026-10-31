import fs from "node:fs";
import path from "node:path";
import { promisify } from "node:util";

/** @type {import('@rspack/test-tools').TMultiCompilerCaseConfig} */
export default {
	description: "should isolate module caches that share the same storage",
	options(context) {
		const cacheLocation = context.getDist("cache");
		const counterFile = context.getDist("module-builds.txt");
		fs.rmSync(cacheLocation, { recursive: true, force: true });
		fs.rmSync(counterFile, { force: true });

		const createOptions = (name, dependencies) => ({
			name,
			dependencies,
			context: import.meta.dirname,
			mode: "development",
			entry: "./module-cache/index.js",
			output: {
				path: context.getDist(name),
				filename: "bundle.js"
			},
			cache: {
				type: "persistent",
				readonly: true,
				maxMemoryGenerations: Infinity,
				storage: {
					type: "filesystem",
					location: cacheLocation
				}
			},
			experiments: {
				newCache: {
					codeGeneration: false,
					devtool: false,
					loader: false,
					minimize: false,
					module: true
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
									"module-cache/compiler-name-loader.mjs"
								),
								options: { counterFile }
							}
						]
					}
				]
			}
		});

		return [createOptions("a"), createOptions("b", ["a"])];
	},
	async build(context, compiler) {
		try {
			const stats = await promisify(compiler.run.bind(compiler))();
			expect(stats.hasErrors()).toBe(false);
			expect(fs.readFileSync(context.getDist("module-builds.txt"), "utf-8")).toBe(
				"ab"
			);

			const [a, b] = stats.stats.map(stat =>
				stat.compilation.getAsset("bundle.js").source.source().toString()
			);
			expect(a).toContain("compiler:a");
			expect(a).not.toContain("compiler:b");
			expect(b).toContain("compiler:b");
			expect(b).not.toContain("compiler:a");
		} finally {
			await promisify(compiler.close.bind(compiler))();
		}
	}
};
