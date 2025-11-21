/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	experiments: {
		outputModule: true,
		css: true
	},
	output: {
		module: true,
		chunkFormat: "module",
		filename: "[name].mjs",
		chunkFilename: "[name].chunk.mjs",
		enabledLibraryTypes: ["module"]
	},
	optimization: {
		minimize: false,
		runtimeChunk: "single"
	}
};
