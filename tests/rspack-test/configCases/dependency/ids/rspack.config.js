class Plugin {
	apply(compiler) {
		compiler.hooks.compilation.tap("Test", compilation => {
			compilation.hooks.finishModules.tap("Test", () => {
				const entry = compilation.entries.get("main");
				const entryDependency = entry.dependencies[0];
				const entryModule = compilation.moduleGraph.getModule(entryDependency);
				const esmImportSpecifierDependency = entryModule.dependencies.find(
					d => d.type === "esm import specifier"
				);
				expect(esmImportSpecifierDependency.ids).toContain("foo");
			});
		});
	}
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	optimization: {
		sideEffects: false
	},
	plugins: [new Plugin()]
};
