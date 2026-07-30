const fs = require("node:fs");
const path = require("node:path");
const rspack = require("@rspack/core");

const CHILD_COMPILER_NAMES = ["child-a", "child-b"];
const CACHE_DIRECTORY_REGEXP = /^rspack_v_[0-9a-f]{16}$/;

class ChildCompilersPlugin {
	apply(compiler) {
		compiler.hooks.make.tapAsync(
			"ChildCompilersPlugin",
			(compilation, callback) => {
				const runChildCompiler = name =>
					new Promise((resolve, reject) => {
						const childCompiler = compilation.createChildCompiler(
							name,
							{ filename: `${name}.js` },
							[
								new compiler.rspack.EntryPlugin(
									compiler.context,
									"./a",
									{ name }
								)
							]
						);

						childCompiler.runAsChild(error => {
							childCompiler.close(closeError => {
								const finalError = error || closeError;
								if (finalError) {
									reject(finalError);
								} else {
									resolve();
								}
							});
						});
					});

				(async () => {
					for (const name of CHILD_COMPILER_NAMES) {
						await runChildCompiler(name);
					}
				})().then(() => callback(), callback);
			}
		);
	}
}

const runCompiler = (context, cacheDirectory, version) =>
	new Promise((resolve, reject) => {
		const compiler = rspack({
			name: "root",
			context: context.getSource(),
			entry: "./a",
			mode: "development",
			output: {
				path: context.getDist(`output-${version}`)
			},
			cache: {
				type: "persistent",
				version,
				storage: {
					type: "filesystem",
					directory: cacheDirectory
				}
			},
			plugins: [new ChildCompilersPlugin()]
		});

		compiler.run(error => {
			compiler.close(closeError => {
				const finalError = error || closeError;
				if (finalError) {
					reject(finalError);
				} else {
					resolve();
				}
			});
		});
	});

/** @type {import("@rspack/test-tools").TCompilerCaseConfig} */
module.exports = {
	description:
		"should keep one persistent cache directory for each compiler path",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./a"
		};
	},
	async build(context) {
		const cacheDirectory = context.getDist(
			"persistent-cache-compiler-path"
		);
		fs.rmSync(cacheDirectory, { recursive: true, force: true });

		await runCompiler(context, cacheDirectory, "v1");
		await runCompiler(context, cacheDirectory, "v2");

		context.setValue(
			"cacheDirectories",
			fs
				.readdirSync(cacheDirectory)
				.filter(name => CACHE_DIRECTORY_REGEXP.test(name))
				.sort()
		);
	},
	check({ context }) {
		const cacheDirectories = context.getValue("cacheDirectories");

		expect(cacheDirectories).toHaveLength(1 + CHILD_COMPILER_NAMES.length);
		expect(new Set(cacheDirectories).size).toBe(cacheDirectories.length);
	}
};
