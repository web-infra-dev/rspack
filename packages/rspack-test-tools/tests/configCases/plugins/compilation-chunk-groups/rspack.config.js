function plugin(compiler) {
	compiler.hooks.compilation.tap("plugin", compilation => {
		compilation.hooks.processAssets.tap("plugin", () => {
			const chunkGroups = compilation.chunkGroups;
			expect(chunkGroups.length).toBe(6);
			expect(chunkGroups.find(i => i.name === 'a').getFiles()).toEqual(['a.js']);
			expect(chunkGroups.find(i => i.name === 'b').getFiles()).toEqual(['b.js']);

			const namedChunkGroups = compilation.namedChunkGroups;
			expect(Array.from(namedChunkGroups.keys()).length).toBe(2);
			expect(namedChunkGroups.get("a").getFiles()).toEqual(['a.js']);
			expect(namedChunkGroups.get("b").getFiles()).toEqual(['b.js']);
		});
	});
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		a: "./entry1.js",
		b: "./entry2.js",
	},
	output: {
		filename: "[name].js",
	},
	plugins: [plugin]
};
