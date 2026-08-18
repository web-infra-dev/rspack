const fs = require("node:fs");
const path = require("node:path");
const rspack = require("@rspack/core");

const hooks = [];
const PLUGIN_NAME = "NewCacheValueDependenciesTestPlugin";

class NewCacheValueDependenciesTestPlugin {
	apply(compiler) {
		compiler.hooks.compilation.tap(PLUGIN_NAME, compilation => {
			for (const hookName of [
				"buildModule",
				"succeedModule",
				"stillValidModule"
			]) {
				compilation.hooks[hookName].tap(PLUGIN_NAME, module => {
					if (module.resource?.endsWith("index.js")) hooks.push(hookName);
				});
			}
		});
	}
}

const runCompiler = (context, cacheDirectory, output, value) =>
	new Promise((resolve, reject) => {
		const compiler = rspack({
			context: context.getSource("new-cache-value-dependencies"),
			entry: "./index.js",
			cache: {
				type: "persistent",
				storage: {
					type: "filesystem",
					location: cacheDirectory
				}
			},
			experiments: {
				newCache: true
			},
			output: {
				path: output
			},
			plugins: [
				new rspack.DefinePlugin({ VALUE: JSON.stringify(value) }),
				new NewCacheValueDependenciesTestPlugin()
			]
		});

		compiler.run((error, stats) => {
			compiler.close(closeError => {
				const finalError = error || closeError;
				if (finalError) return reject(finalError);
				if (stats.hasErrors()) {
					return reject(
						new Error(stats.toString({ all: false, errors: true }))
					);
				}
				resolve();
			});
		});
	});

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
module.exports = {
	description: "should invalidate the new cache when value dependencies change",
	options(context) {
		return {
			context: context.getSource("new-cache-value-dependencies"),
			entry: "./index.js"
		};
	},
	async build(context) {
		const cacheDirectory = context.getDist(
			"new-cache-value-dependencies-cache"
		);
		fs.rmSync(cacheDirectory, { recursive: true, force: true });

		await runCompiler(context, cacheDirectory, context.getDist("output-1"), "first");
		await runCompiler(context, cacheDirectory, context.getDist("output-2"), "first");
		const output = context.getDist("output-3");
		await runCompiler(context, cacheDirectory, output, "second");

		expect(fs.readFileSync(path.join(output, "main.js"), "utf-8")).toContain(
			"second"
		);
	},
	check() {
		expect(hooks).toEqual([
			"buildModule",
			"succeedModule",
			"stillValidModule",
			"buildModule",
			"succeedModule"
		]);
	}
};
