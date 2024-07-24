/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	target: "web",
	devtool: false,
	output: {
		assetModuleFilename: "[name][ext]"
	},
	module: {
		rules: [
			{
				test: /\.css$/,
				dependency: "url",
				loader: "url-loader"
			}
		]
	}
};
