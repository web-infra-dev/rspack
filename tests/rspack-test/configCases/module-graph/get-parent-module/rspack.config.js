class Plugin {
	apply(compiler) {
		compiler.hooks.compilation.tap("Test", compilation => {
			compilation.hooks.processAssets.tap("Test", () => {
				const entry = compilation.entries.get("main");
				const entryDependency = entry.dependencies[0];
				const entryModule = compilation.moduleGraph.getModule(entryDependency);
				expect(
					compilation.moduleGraph.getParentModule(entryModule.dependencies[0])
				).toBe(entryModule);
			});
		});
	}
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "web",
	node: false,
	entry: {
		main: "./index.js"
	},
	optimization: {
		sideEffects: false
	},
	plugins: [new Plugin()]
};
