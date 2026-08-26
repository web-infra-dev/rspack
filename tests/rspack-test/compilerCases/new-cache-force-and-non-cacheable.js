const path = require("node:path");
const { createFsFromVolume, Volume } = require("memfs");

const hooks = [];
let compilationCount = 0;
const PLUGIN_NAME = "NewCacheForceAndNonCacheableTestPlugin";

class NewCacheForceAndNonCacheableTestPlugin {
	apply(compiler) {
		compiler.hooks.compilation.tap(PLUGIN_NAME, compilation => {
			compilationCount++;
			for (const hookName of [
				"buildModule",
				"succeedModule",
				"stillValidModule"
			]) {
				compilation.hooks[hookName].tap(PLUGIN_NAME, module => {
					if (module.resource?.endsWith("index.js")) hooks.push(hookName);
				});
			}
			compilation.hooks.finishModules.tapAsync(
				PLUGIN_NAME,
				(modules, callback) => {
					if (compilationCount !== 2) return callback();
					const module = [...modules].find(item =>
						item.resource?.endsWith("index.js")
					);
					if (!module) return callback(new Error("module not found"));
					compilation.rebuildModule(module, callback);
				}
			);
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
	description:
		"should force rebuild cached modules and replace old entries when cacheability changes",
	options(context) {
		return {
			context: context.getSource("new-cache-non-cacheable"),
			entry: "./index.js",
			cache: true,
			incremental: false,
			experiments: {
				newCache: {
					module: true,
					codeGeneration: false,
					devtool: false,
					loader: false,
					minimize: false
				}
			},
			module: {
				rules: [
					{
						test: /index\.js$/,
						use: path.resolve(context.getSource("new-cache-non-cacheable/loader.js"))
					}
				]
			},
			output: { path: "/directory" },
			plugins: [new NewCacheForceAndNonCacheableTestPlugin()]
		};
	},
	compiler(_, compiler) {
		compiler.outputFileSystem = createFsFromVolume(new Volume());
	},
	async build(context, compiler) {
		const loader = require(context.getSource("new-cache-non-cacheable/loader.js"));
		loader.setCacheable(true);
		try {
			await run(compiler);
			loader.setCacheable(false);
			await run(compiler);
			await run(compiler);
		} finally {
			loader.setCacheable(true);
		}
	},
	check() {
		expect(hooks).toEqual([
			"buildModule",
			"succeedModule",
			"stillValidModule",
			"buildModule",
			"succeedModule",
			"buildModule",
			"succeedModule"
		]);
	}
};
