import fs from "node:fs";
import path from "node:path";
import { promisify } from "node:util";

/** @type {import('@rspack/test-tools').TMultiCompilerCaseConfig} */
export default {
	description: "should reuse the loader cache across compilers",
	options(context) {
		const cacheLocation = context.getDist("cache");
		const counterFile = context.getDist("loader-runs.txt");
		fs.rmSync(cacheLocation, { recursive: true, force: true });
		fs.rmSync(counterFile, { force: true });

		const createOptions = (name, dependencies) => ({
			name,
			dependencies,
			context: import.meta.dirname,
			mode: "development",
			entry: "./loader-cache/index.js",
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
		try {
			const stats = await promisify(compiler.run.bind(compiler))();
			expect(stats.hasErrors()).toBe(false);
			expect(fs.readFileSync(context.getDist("loader-runs.txt"), "utf-8")).toBe(
				"x"
			);
		} finally {
			await promisify(compiler.close.bind(compiler))();
		}
	}
};
