/* global document */

const { CssExtractRspackPlugin } = require("@rspack/core");

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
	optimization: {
		chunkIds: "named"
	},
	plugins: [
		new CssExtractRspackPlugin({
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
