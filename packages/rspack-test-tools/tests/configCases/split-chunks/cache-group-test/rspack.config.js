/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	target: "node",
	output: {
		filename: "[name].js"
	},
	optimization: {
		splitChunks: {
			cacheGroups: {
				common: {
					test(module, { moduleGraph, chunkGraph }) {
						expect(module.size()).toBe(5);
						expect(moduleGraph.isAsync(module)).toBe(false);
						expect(chunkGraph.getModuleChunks(module).length).toBe(1);
						expect(
							Array.from(chunkGraph.getModuleChunksIterable(module)).length
						).toBe(1);
						return true;
					}
				}
			}
		}
	}
};
