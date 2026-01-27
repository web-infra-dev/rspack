/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	target: "web",
	experiments: {
		outputModule: true
	},
	module: {
		rules: [
			{
				test: /\.css$/,
				type: "css/module"
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
		minimize: false
	}
};
