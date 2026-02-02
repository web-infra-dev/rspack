/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.mjs",
	module: {
		rules: [
			{
				test: /\.css$/,
				type: "css/auto"
			}
		]
	},
	name: "esm",
	target: "web",
	output: {
		module: true,
		publicPath: "",
		module: true,
		filename: "bundle0.mjs",
		chunkFilename: "[name].mjs",
		crossOriginLoading: "anonymous",
		chunkFormat: "module"
	},
	performance: {
		hints: false
	},
	optimization: {
		minimize: false
	}
};
