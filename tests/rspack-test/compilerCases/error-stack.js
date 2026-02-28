let error;

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
module.exports = [{
	description: "should print error with stack information with sync callback",
	error: true,
	options(context) {
		return {
			context: context.getSource(),
			entry: "./a",
			output: {
				filename: "bundle.js"
			},
			plugins: [{
				apply(compiler) {
					compiler.hooks.compilation.tap("MyPlugin", compilation => {
						throw new Error("Failed to handle process assets from JS");
					});
				}
			}]
		};
	},
	async build(_, compiler) {
		await new Promise(resolve => {
			compiler.run((err, _) => {
				error = err;
				resolve();
			});
		});
	},
	async check() {
		expect(error).toBeTruthy();
		expect(error.message).toContain("Failed to handle process assets from JS");
	}
}, {
	description: "should print error with stack information with async callback",
	error: true,
	options(context) {
		return {
			context: context.getSource(),
			entry: "./a",
			output: {
				filename: "bundle.js"
			},
			plugins: [{
				apply(compiler) {
					compiler.hooks.compilation.tap("MyPlugin", compilation => {
						compilation.hooks.processAssets.tapPromise("MyPlugin", async assets => {
							throw new Error("Failed to handle process assets from JS");
						});
					});
				}
			}]
		};
	},
	async build(_, compiler) {
		await new Promise(resolve => {
			compiler.run((err, _) => {
				error = err;
				resolve();
			});
		});
	},
	async check() {
		expect(error).toBeTruthy();
		expect(error.message).toContain("Failed to handle process assets from JS");
	}
}];
