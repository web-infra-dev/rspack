/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	target: "node",
	entry: {
		main: "./index.js"
	},
	output: {
		module: true,
		chunkLoading: "import",
		chunkFormat: "module",
		filename: "[name].mjs",
		chunkFilename: "[name].chunk.js"
	},
	optimization: {
		runtimeChunk: true,
		minimize: false
	}
};
