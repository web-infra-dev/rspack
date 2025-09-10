class MyPlugin {
	apply(compiler) {
		compiler.hooks.compilation.tap("MyPlugin", compilation => {
			compilation.hooks.optimizeModules.tap("MyPlugin", modules => {
				modules = [...modules]
				expect(modules.length).toEqual(1);
				expect(modules[0].resource.includes("d.js")).toBeTruthy();
			});
		});
	}
}

/** @type {import('../..').TCompilerCaseConfig} */
module.exports = {
	description: "should call optimizeModules hook correctly",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./d",
			plugins: [new MyPlugin()]
		};
	}
};
