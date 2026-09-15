const path = require("path");
const { createFsFromVolume, Volume } = require("memfs");

let buildError;
let buildStats;
let injectedError;
let outputFileSystem;

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
module.exports = {
	description: "should report EMFILE from the output file system during emit",
	options(context) {
		return {
			entry: "./a",
			output: {
				path: context.getDist(),
				filename: "main.js"
			}
		};
	},
	compiler(context, compiler) {
		buildError = undefined;
		buildStats = undefined;
		injectedError = undefined;
		outputFileSystem = createFsFromVolume(new Volume());
		const writeFile = outputFileSystem.writeFile.bind(outputFileSystem);

		outputFileSystem.writeFile = (filename, content, callback) => {
			if (!injectedError && filename.endsWith("main.js")) {
				injectedError = new Error(
					`EMFILE: too many open files, open '${filename}'`
				);
				injectedError.code = "EMFILE";
				process.nextTick(() => callback(injectedError));
				return;
			}

			writeFile(filename, content, callback);
		};

		compiler.outputFileSystem = outputFileSystem;
	},
	build(context, compiler) {
		return new Promise(resolve => {
			compiler.run((error, stats) => {
				buildError = error;
				buildStats = stats;
				resolve();
			});
		});
	},
	check({ context }) {
		expect(buildError).toBeTruthy();
		expect(buildError.message).toContain("EMFILE");
		expect(injectedError).toBeTruthy();
		expect(injectedError.code).toBe("EMFILE");
		expect(buildStats).toBeFalsy();
		expect(
			outputFileSystem.existsSync(path.join(context.getDist(), "main.js"))
		).toBe(false);
	}
};
