class MyPlugin {
	apply(compiler) {
		compiler.hooks.compilation.tap("MyPlugin", compilation => {
			let cache = compilation.getCache("MyPlugin");
			expect(cache).not.toBeNull();
		});
	}
}

/** @type {import('../..').TCompilerCaseConfig} */
module.exports = {
	description: "should call getCache function correctly",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./d",
			plugins: [new MyPlugin()]
		};
	}
};
