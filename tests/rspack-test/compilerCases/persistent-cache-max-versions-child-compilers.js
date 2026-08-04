const fs = require("node:fs");
const path = require("node:path");
const rspack = require("@rspack/core");

const CHILD_COMPILER_NAMES = ["child-a", "child-b"];
const VERSION_DIRECTORY_REGEXP =
	/^rspack_v_([0-9a-f]{16})_[0-9a-f]{16}$/;

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
				maxVersions: 2,
				storage: {
					type: "filesystem",
					location: cacheDirectory
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

const getVersionsByScope = cacheDirectory => {
	const versionsByScope = new Map();
	for (const entry of fs.readdirSync(cacheDirectory)) {
		const match = VERSION_DIRECTORY_REGEXP.exec(entry);
		if (!match) continue;

		const versions = versionsByScope.get(match[1]) || [];
		versions.push(entry);
		versionsByScope.set(match[1], versions);
	}
	return versionsByScope;
};

/** @type {import("@rspack/test-tools").TCompilerCaseConfig} */
module.exports = {
	description:
		"should retain maxVersions for the root and each named child compiler",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./a"
		};
	},
	async build(context) {
		const cacheDirectory = context.getDist("persistent-cache-child-compilers");
		fs.rmSync(cacheDirectory, { recursive: true, force: true });

		for (const version of ["v1", "v2", "v3"]) {
			await runCompiler(context, cacheDirectory, version);
		}

		context.setValue("versionsByScope", getVersionsByScope(cacheDirectory));
	},
	check({ context }) {
		const versionsByScope = context.getValue("versionsByScope");

		expect(versionsByScope.size).toBe(1 + CHILD_COMPILER_NAMES.length);
		expect(
			Array.from(versionsByScope.values(), versions => versions.length)
		).toEqual([2, 2, 2]);
	}
};
