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
	optimization: {
		chunkIds: "named"
	},
	plugins: [
		new RspackCssExtractPlugin({
			filename: "[name].css",
			chunkFilename: "[id].css",
			// eslint-disable-next-line
			insert: function (linkTag) {
				const reference = document.querySelector(".hot-reload");
				if (reference) {
					reference.parentNode.insertBefore(linkTag, reference);
				}
			}
		})
	]
};
