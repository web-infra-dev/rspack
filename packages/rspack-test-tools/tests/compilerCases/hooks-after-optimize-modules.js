class MyPlugin {
	apply(compiler) {
		let a = 1;
		compiler.hooks.compilation.tap("MyPlugin", compilation => {
			compilation.hooks.optimizeModules.tap("MyPlugin", () => {
				a += 1;
			});

			compilation.hooks.afterOptimizeModules.tap("MyPlugin", modules => {
				expect(a).toBeGreaterThan(1);
				expect(modules.length).toEqual(1);
				expect(modules[0].resource.includes("d.js")).toBeTruthy();
			});
		});
	}
}

module.exports = {
	description: "should call afterOptimizeModules hook correctly",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./d",
			plugins: [new MyPlugin()]
		};
	}
};
