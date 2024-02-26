/* global document */

import { RspackCssExtractPlugin } from "../../../../src";

module.exports = {
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [
					{
						loader: RspackCssExtractPlugin.loader
					},
					{
						loader: "css-loader"
					}
				]
			}
		]
	},
	plugins: [
		new RspackCssExtractPlugin({
			filename: "[name].css",
			chunkFilename: "[id].css"
		})
	]
};
