const fs = require("node:fs");
const rspack = require("@rspack/core");

const hooks = [];
const PLUGIN_NAME = "NewCachePersistentNormalModuleTestPlugin";

class NewCachePersistentNormalModuleTestPlugin {
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

const runCompiler = (context, cacheDirectory, output) =>
	new Promise((resolve, reject) => {
		const compiler = rspack({
			context: context.getSource("new-cache-normal-module"),
			entry: "./index.js",
			cache: {
				type: "persistent",
				storage: {
					type: "filesystem",
					location: cacheDirectory
				}
			},
			incremental: true,
			experiments: {
				newCache: true
			},
			output: {
				path: output
			},
			plugins: [new NewCachePersistentNormalModuleTestPlugin()]
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
	description:
		"should restore unchanged NormalModule builds from the new persistent cache",
	options(context) {
		return {
			context: context.getSource("new-cache-normal-module"),
			entry: "./index.js"
		};
	},
	async build(context) {
		const cacheDirectory = context.getDist(
			"new-cache-persistent-normal-module-cache"
		);
		fs.rmSync(cacheDirectory, { recursive: true, force: true });

		await runCompiler(context, cacheDirectory, context.getDist("output-1"));
		await runCompiler(context, cacheDirectory, context.getDist("output-2"));
	},
	check() {
		expect(hooks).toEqual([
			"buildModule",
			"succeedModule",
			"stillValidModule"
		]);
	}
};
