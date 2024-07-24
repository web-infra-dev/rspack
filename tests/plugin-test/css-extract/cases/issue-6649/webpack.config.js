/* global document */

const { CssExtractRspackPlugin } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		"\\entry\\name": "./src/index.js"
	},
	optimization: {
		chunkIds: "named"
	},
	output: {
		chunkFilename: "[name].$[contenthash]$.js",
		filename: "main.js"
	},
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [
					{
						loader: CssExtractRspackPlugin.loader
					},
					{
						loader: "css-loader"
					}
				]
			}
		]
	},
	plugins: [
		new CssExtractRspackPlugin({
			filename: "[name].$[contenthash]$.css",
		})
	]
};
