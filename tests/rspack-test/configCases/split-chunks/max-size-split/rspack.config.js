/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "node",
	entry: "./src/index.js",
	output: {
		filename: "[name].js"
	},
	optimization: {
		chunkIds: "named",
		moduleIds: "named",
		splitChunks: {
			chunks: "all",
			cacheGroups: {
				fragment: {
					minChunks: 1,
					maxSize: 200 * 1024,
					priority: 10
				}
			}
		}
	}
};
