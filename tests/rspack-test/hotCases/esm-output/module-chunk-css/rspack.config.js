/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	experiments: {
		outputModule: true
	},
	module: {
		rules: [
			{
				test: /\.css$/,
				type: "css/auto"
			}
		]
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
