class Plugin {
	apply(compiler) {
		compiler.hooks.compilation.tap("Test", compilation => {
			compilation.hooks.processAssets.tap("Test", () => {
				const entry = compilation.entries.get("main");
				const entryDependency = entry.dependencies[0];
				const entryModule = compilation.moduleGraph.getModule(entryDependency);

				const fooDependency = entryModule.dependencies.find(
					dep => dep.request === "./foo"
				);
				expect(compilation.moduleGraph.getParentBlockIndex(fooDependency)).toBe(
					0
				);

				const barDependency = entryModule.dependencies.find(
					dep => dep.request === "./bar"
				);
				expect(compilation.moduleGraph.getParentBlockIndex(barDependency)).toBe(
					1
				);
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
	plugins: [new Plugin()]
};
