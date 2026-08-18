const fs = require("node:fs");
const path = require("node:path");
const { createFsFromVolume, Volume } = require("memfs");

const hooks = [];
const PLUGIN_NAME = "NewCacheNormalModuleIncrementalFalseTestPlugin";

class NewCacheNormalModuleIncrementalFalseTestPlugin {
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
	description:
		"should use the new fine-grained cache when incremental compilation is disabled",
	options(context) {
		return {
			context: context.getSource("new-cache-normal-module-incremental-false"),
			entry: "./index.js",
			cache: true,
			incremental: false,
			experiments: {
				newCache: true
			},
			output: {
				path: "/directory"
			},
			plugins: [new NewCacheNormalModuleIncrementalFalseTestPlugin()]
		};
	},
	compiler(_, compiler) {
		compiler.outputFileSystem = createFsFromVolume(new Volume());
	},
	async build(context, compiler) {
		const source = context.getSource(
			"new-cache-normal-module-incremental-false/index.js"
		);
		const original = fs.readFileSync(source, "utf-8");

		try {
			await run(compiler);
			await run(compiler);
			fs.writeFileSync(source, 'export default "changed";\n');
			await run(compiler);
		} finally {
			fs.writeFileSync(source, original);
		}

		const output = compiler.outputFileSystem.readFileSync(
			path.posix.join("/directory", "main.js"),
			"utf-8"
		);
		expect(output).toContain("changed");
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
