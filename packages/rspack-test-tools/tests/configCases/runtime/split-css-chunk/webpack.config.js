/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		main: "./index1.js",
		main2: "./index2.js"
	},
	output: {
		chunkFilename: "[id].[contenthash].js",
		filename: '[name].js'
	},
	optimization: {
		splitChunks: {
			cacheGroups: {
				common: {
					chunks: "all",
					test: /common/,
					enforce: true,
					name: "common"
				}
			}
		},
		runtimeChunk: "single"
	}
};
