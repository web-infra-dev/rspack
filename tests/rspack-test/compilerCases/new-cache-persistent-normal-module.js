const fs = require("node:fs");
const path = require("node:path");
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
			mode: "production",
			context: context.getSource("new-cache-normal-module"),
			entry: "./index.js",
			cache: {
				type: "persistent",
				storage: { type: "filesystem", location: cacheDirectory }
			},
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
			optimization: { minimize: false },
			output: { path: output },
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
		"should restore persistent module builds and keep dependency ids unique",
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
		const source = context.getSource("new-cache-normal-module/index.js");
		const original = fs.readFileSync(source, "utf-8");
		const originalTimes = fs.statSync(source);
		fs.rmSync(cacheDirectory, { recursive: true, force: true });

		try {
			await runCompiler(context, cacheDirectory, context.getDist("output-1"));
			await runCompiler(
				context,
				cacheDirectory,
				context.getDist("output-2")
			);

			// A production snapshot falls back to hash when only mtime changed.
			const touched = new Date(Date.now() + 2_000);
			fs.utimesSync(source, touched, touched);
			await runCompiler(
				context,
				cacheDirectory,
				context.getDist("output-3")
			);

			fs.writeFileSync(
				source,
				'import value from "./dep";\nimport fresh from "./new";\nconsole.log(value, fresh);\n'
			);
			const changed = new Date(Date.now() + 4_000);
			fs.utimesSync(source, changed, changed);
			const output = context.getDist("output-4");
			await runCompiler(context, cacheDirectory, output);
			const bundle = fs.readFileSync(path.join(output, "main.js"), "utf-8");
			expect(bundle).toContain("cached-");
			expect(bundle).toContain("new-dependency");
		} finally {
			fs.writeFileSync(source, original);
			fs.utimesSync(source, originalTimes.atime, originalTimes.mtime);
		}
	},
	check() {
		expect(hooks).toEqual([
			"buildModule",
			"succeedModule",
			"stillValidModule",
			"stillValidModule",
			"buildModule",
			"succeedModule"
		]);
	}
};
