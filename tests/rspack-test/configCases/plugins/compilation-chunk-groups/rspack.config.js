function plugin(compiler) {
	compiler.hooks.compilation.tap("plugin", compilation => {
		compilation.hooks.processAssets.tap("plugin", () => {
			const chunkGroups = compilation.chunkGroups;
			expect(chunkGroups.length).toBe(6);
			expect(chunkGroups.find(i => i.name === "a").getFiles()).toEqual([
				"a.js"
			]);
			expect(chunkGroups.find(i => i.name === "b").getFiles()).toEqual([
				"b.js"
			]);
			expect(chunkGroups.filter(i => i.isInitial()).length).toEqual(2);

			const namedChunkGroups = compilation.namedChunkGroups;
			expect(Array.from(namedChunkGroups.keys()).length).toBe(2);
			expect(namedChunkGroups.get("a").getFiles()).toEqual(["a.js"]);
			expect(namedChunkGroups.get("b").getFiles()).toEqual(["b.js"]);

			// for of
			const result1 = [];
			for (const [key, value] of namedChunkGroups) {
				result1.push([key, value.getFiles()]);
			}
			result1.sort(([a], [b]) => (a > b ? 1 : -1));
			expect(result1).toEqual([
				["a", ["a.js"]],
				["b", ["b.js"]]
			]);

			// forEach
			const result2 = [];
			namedChunkGroups.forEach((value, key) => {
				result2.push([key, value.getFiles()]);
			});
			result2.sort(([a], [b]) => (a > b ? 1 : -1));
			expect(result2).toEqual([
				["a", ["a.js"]],
				["b", ["b.js"]]
			]);

			// values
			const result3 = [];
			for (const value of namedChunkGroups.values()) {
				result3.push(value.getFiles());
			}
			result3.sort(([a], [b]) => (a[0] > b[0] ? 1 : -1));
			expect(result3).toEqual([["a.js"], ["b.js"]]);

			const origins = chunkGroups.reduce((res, i) => {
				res.push(...i.origins.map(i => i.module?.rawRequest).filter(Boolean));
				return res;
			}, []);
			origins.sort();
			expect(origins).toEqual([
				"./entry1.js",
				"./entry1.js",
				"./entry2.js",
				"./entry2.js"
			]);
		});
	});
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		a: "./entry1.js",
		b: "./entry2.js"
	},
	output: {
		filename: "[name].js"
	},
	plugins: [plugin]
};
