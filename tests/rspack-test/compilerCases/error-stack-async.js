class MyPlugin {
	apply(compiler) {
		compiler.hooks.compilation.tap("MyPlugin", compilation => {
			compilation.hooks.processAssets.tapPromise("MyPlugin", async assets => {
				throw new Error("Failed to handle process assets from JS");
			});
		});
	}
}

let error;

/** @type {import('../..').TCompilerCaseConfig} */
module.exports = {
	description: "should print error with stack information with async callback",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./a",
			output: {
				filename: "bundle.js"
			},
			plugins: [new MyPlugin()]
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
};
