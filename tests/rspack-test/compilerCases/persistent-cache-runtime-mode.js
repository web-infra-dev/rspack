const fs = require("node:fs");
const path = require("node:path");
const rspack = require("@rspack/core");

const compile = (
	workDir,
	cacheDirectory,
	outputDirectory,
	runtimeMode
) =>
	new Promise((resolve, reject) => {
		const compiler = rspack({
			context: workDir,
			entry: "./index.js",
			mode: "development",
			target: "node",
			devtool: false,
			cache: {
				type: "persistent",
				buildDependencies: [__filename],
				storage: {
					type: "filesystem",
					directory: cacheDirectory
				}
			},
			experiments: {
				runtimeMode
			},
			output: {
				path: outputDirectory,
				filename: "main.js"
			}
		});

		compiler.run((error, stats) => {
			compiler.close(closeError => {
				const finalError = error || closeError;
				if (finalError) {
					reject(finalError);
				} else if (stats.hasErrors()) {
					reject(new Error(stats.toString({ all: false, errors: true })));
				} else {
					resolve();
				}
			});
		});
	});

/** @type {import("@rspack/test-tools").TCompilerCaseConfig} */
module.exports = {
	description:
		"should render the HMR interceptor after recovering a persistent cache from another runtime mode",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./a"
		};
	},
	async build(context) {
		const workDir = context.getDist("persistent-cache-runtime-mode-workdir");
		const cacheDirectory = context.getDist("persistent-cache-runtime-mode");
		const outputDirectory = context.getDist(
			"persistent-cache-runtime-mode-output"
		);
		fs.rmSync(workDir, { recursive: true, force: true });
		fs.rmSync(cacheDirectory, { recursive: true, force: true });
		fs.mkdirSync(workDir, { recursive: true });
		fs.writeFileSync(
			path.join(workDir, "index.js"),
			[
				"const interceptors = __webpack_require__.i;",
				"__webpack_require__.i.push(function intercept() {});",
				"module.exports = 42;"
			].join("\n")
		);

		await compile(
			workDir,
			cacheDirectory,
			outputDirectory,
			"webpack"
		);
		context.setValue(
			"webpackOutput",
			fs.readFileSync(path.join(outputDirectory, "main.js"), "utf-8")
		);

		await compile(workDir, cacheDirectory, outputDirectory, "rspack");
		context.setValue(
			"rspackOutput",
			fs.readFileSync(path.join(outputDirectory, "main.js"), "utf-8")
		);
	},
	check({ context }) {
		const webpackOutput = context.getValue("webpackOutput");
		const rspackOutput = context.getValue("rspackOutput");

		expect(webpackOutput).toContain("__webpack_require__.i.push");
		expect(rspackOutput).toContain(
			"const interceptors = __rspack_context.i;"
		);
		expect(rspackOutput).toContain("__rspack_context.i.push");
		expect(rspackOutput).not.toContain("__webpack_require__.i.push");
		expect(() =>
			require(
				context.getDist(
					path.join("persistent-cache-runtime-mode-output", "main.js")
				)
			)
		).not.toThrow();
	}
};
