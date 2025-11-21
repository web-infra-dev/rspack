/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		a: "./a/index.js",
		b: "./b/index.js",
		main: "./main/index.js"
	},
	output: {
		clean: false,
		filename: "[name].[chunkhash].[contenthash].js",
		chunkFilename: "[name].[chunkhash].[contenthash].js"
	},
	optimization: {
		runtimeChunk: "single"
	}
};
