
/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
module.exports = [{
	description: "should call optimizeModules hook correctly",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./d",
			plugins: [{
				apply(compiler) {
					compiler.hooks.compilation.tap("MyPlugin", compilation => {
						compilation.hooks.optimizeModules.tap("MyPlugin", modules => {
							modules = [...modules]
							expect(modules.length).toEqual(1);
							expect(modules[0].resource.includes("d.js")).toBeTruthy();
						});
					});
				}
			}]
		};
	}
}, {
	description: "should call afterOptimizeModules hook correctly",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./d",
			plugins: [{
				apply(compiler) {
					let a = 1;
					compiler.hooks.compilation.tap("MyPlugin", compilation => {
						compilation.hooks.optimizeModules.tap("MyPlugin", () => {
							a += 1;
						});

						compilation.hooks.afterOptimizeModules.tap("MyPlugin", modules => {
							expect(a).toBeGreaterThan(1);
							modules = [...modules];
							expect(modules.length).toEqual(1);
							expect(modules[0].resource.includes("d.js")).toBeTruthy();
						});
					});
				}
			}]
		};
	}
}];
