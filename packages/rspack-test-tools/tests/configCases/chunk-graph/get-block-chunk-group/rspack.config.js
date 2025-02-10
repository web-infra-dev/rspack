class Plugin {
	apply(compiler) {
		compiler.hooks.compilation.tap("Test", compilation => {
			compilation.hooks.processAssets.tap("Test", () => {
				const entry = compilation.entries.get("main");
				const entryDependency = entry.dependencies[0];
				const entryModule = compilation.moduleGraph.getModule(entryDependency);
				const block = entryModule.blocks[0];
				const chunkGroup = compilation.chunkGraph.getBlockChunkGroup(block);
				expect(chunkGroup.name).toBe("foo");
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
	output: {
		filename: "[name].js"
	},
	optimization: {
		sideEffects: false
	},
	plugins: [new Plugin()]
};
