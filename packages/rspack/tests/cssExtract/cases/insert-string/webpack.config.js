/* global document */

const { CssExtractRspackPlugin } = require("../../../../");

module.exports = {
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
			filename: "[name].css",
			chunkFilename: "[id].css",
			insert: "script[src='1.js']"
		})
	]
};
