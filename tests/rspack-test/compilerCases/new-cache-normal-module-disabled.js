const { createFsFromVolume, Volume } = require("memfs");

const hooks = [];
const PLUGIN_NAME = "NewCacheNormalModuleDisabledTestPlugin";

class NewCacheNormalModuleDisabledTestPlugin {
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

const run = compiler =>
	new Promise((resolve, reject) => {
		compiler.run((error, stats) => {
			if (error) return reject(error);
			if (stats.hasErrors()) {
				return reject(new Error(stats.toString({ all: false, errors: true })));
			}
			resolve();
		});
	});

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
module.exports = {
	description: "should not use the new module cache when it is disabled",
	options(context) {
		return {
			context: context.getSource("new-cache-normal-module-incremental-false"),
			entry: "./index.js",
			cache: true,
			incremental: false,
			experiments: {
				newCache: {
					module: false,
					codeGeneration: true,
					devtool: false,
					loader: false,
					minimize: false
				}
			},
			output: { path: "/directory" },
			plugins: [new NewCacheNormalModuleDisabledTestPlugin()]
		};
	},
	compiler(_, compiler) {
		compiler.outputFileSystem = createFsFromVolume(new Volume());
	},
	async build(_, compiler) {
		await run(compiler);
		await run(compiler);
	},
	check() {
		expect(hooks).toEqual([
			"buildModule",
			"succeedModule",
			"buildModule",
			"succeedModule"
		]);
	}
};
