const path = require("node:path");
const rspack = require("@rspack/core");

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
module.exports = [{
	description: "should resolve relative output.path against context",
	options(context) {
		return {
			context: context.getSource(),
			output: {
				path: "subdir/dist"
			}
		};
	},
	async check({ context, compiler, files }) {
		const expectedOutputPath = path.join(context.getSource(), "subdir/dist");

		expect(compiler.options.output.path).toBe(expectedOutputPath);
		expect(compiler.outputPath).toBe(expectedOutputPath);
		expect(Object.keys(files)).toEqual([
			path.join(expectedOutputPath, "main.js")
		]);
	}
}, {
	description: "should resolve relative output.path against default context",
	options(context) {
		return {
			context: context.getSource(),
			output: {
				path: context.getDist()
			}
		};
	},
	async check({ context }) {
		const compiler = rspack({
			entry: context.getSource("a.js"),
			output: {
				path: "dist"
			}
		});

		expect(compiler.options.context).toBe(process.cwd());
		expect(compiler.options.output.path).toBe(path.join(process.cwd(), "dist"));

		await new Promise((resolve, reject) => {
			compiler.close(error => {
				if (error) {
					reject(error);
					return;
				}
				resolve();
			});
		});
	}
}, {
	description: "should throw when context is empty and output.path is relative",
	async build(_, compiler) {
		await new Promise(resolve => {
			compiler.run(() => {
				resolve();
			});
		});
	},
	options() {
		return {
			context: "",
			entry: "./a.js",
			output: {
				path: "subdir/dist"
			}
		};
	},
	async check({ context }) {
		const errors = context.getError();
		expect(Array.isArray(errors)).toBeTruthy();
		expect(errors.length).toBe(1);
		expect(errors[0].toString()).toContain(
			'Invalid Rspack configuration: "context" must be a non-empty absolute path when "output.path" is relative, get "".'
		);
		context.clearError();
	}
}];
