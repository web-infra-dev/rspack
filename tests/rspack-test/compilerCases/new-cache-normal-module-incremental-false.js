const fs = require("node:fs");
const path = require("node:path");
const { createFsFromVolume, Volume } = require("memfs");

const hooks = [];
let sourceReads = 0;
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
		"should cache NormalModule builds independently from incremental compilation",
	options(context) {
		return {
			mode: "development",
			context: context.getSource("new-cache-normal-module-incremental-false"),
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
			output: { path: "/directory" },
			plugins: [new NewCacheNormalModuleIncrementalFalseTestPlugin()]
		};
	},
	compiler(_, compiler) {
		compiler.outputFileSystem = createFsFromVolume(new Volume());
		const inputFileSystem = compiler.inputFileSystem;
		const readFile = inputFileSystem.readFile.bind(inputFileSystem);
		inputFileSystem.readFile = (filename, ...args) => {
			if (String(filename).endsWith("index.js")) sourceReads++;
			return readFile(filename, ...args);
		};
	},
	async build(context, compiler) {
		const source = context.getSource(
			"new-cache-normal-module-incremental-false/index.js"
		);
		const original = fs.readFileSync(source, "utf-8");
		const originalTimes = fs.statSync(source);

		try {
			await run(compiler);
			sourceReads = 0;
			await run(compiler);
			expect(sourceReads).toBe(0);
			const touched = new Date(Date.now() + 2_000);
			fs.utimesSync(source, touched, touched);
			await run(compiler);
			fs.writeFileSync(source, 'export default "changed";\n');
			const changed = new Date(Date.now() + 4_000);
			fs.utimesSync(source, changed, changed);
			await run(compiler);
		} finally {
			fs.writeFileSync(source, original);
			fs.utimesSync(source, originalTimes.atime, originalTimes.mtime);
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
			"succeedModule",
			"buildModule",
			"succeedModule"
		]);
	}
};
