class Plugin {
	apply(compiler) {
		compiler.hooks.compilation.tap("Test", compilation => {
			compilation.hooks.processAssets.tap("Test", () => {
				const entry = compilation.entries.get("main");
				const entryDependency = entry.dependencies[0];
				const entryModule = compilation.moduleGraph.getModule(entryDependency);
				const block = entryModule.blocks[0];
				const chunkGroup = compilation.chunkGraph.getBlockChunkGroup(block);

				expect(chunkGroup.name).toBe("foo-blocks");

				const blocks = compilation.chunkGraph.getChunkGroupBlocks(chunkGroup);
				expect(blocks.length).toBe(1);
				const blockFromGroup = blocks[0];
				expect(blockFromGroup).toBe(block);

				const blockDependencies = block.dependencies;
				expect(blockDependencies.length).toBe(1);
				expect(blockDependencies[0].request).toBe("./foo");

				const groupBlockDependencies = blockFromGroup.dependencies;
				expect(groupBlockDependencies.length).toBe(1);
				expect(groupBlockDependencies[0]).toBe(blockDependencies[0]);
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
