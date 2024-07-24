/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		main: "./index"
	},
	output: {
		filename: "[name].js"
	},
	optimization: {
		splitChunks: {
			chunks: "all",
			minSize: 0,
			cacheGroups: {
				foo: {
					test: /\.js/,
					name(module, chunks, cacheGroupKey) {
						expect(chunks.length).toBeGreaterThan(0);
						expect(cacheGroupKey).toBe("foo");
					}
				}
			}
		}
	}
};
