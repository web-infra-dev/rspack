
let resolveCompilerStats;
let compilerStats = new Promise((resolve) => {
	resolveCompilerStats = resolve
});

class MyPlugin {
	apply(compiler) {
		compiler.hooks.done.tap("Plugin", stats => {
			resolveCompilerStats(stats)
		});
	}
}

/** @type {import('../../dist').TCompilerCaseConfig} */
module.exports = {
	description: "should be called every compilation",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./d",
			plugins: [new MyPlugin()]
		};
	},
	async build(_, compiler) {
		await new Promise(resolve => {
			compiler.run((err, stats) => {
				compiler.close(() => {
					// Should be able to access `Stats` within the same tick of closing.
					expect(() => stats.compilation).not.toThrow();
					resolve()
				})
			});
		});
	},
	async check() {
		let stats = await compilerStats;
		// Should not be able to access `Stats` after the compiler was shutdown.
		expect(() => stats.compilation).toThrow("Unable to access `Stats` after the compiler was shutdown")
	}
};
