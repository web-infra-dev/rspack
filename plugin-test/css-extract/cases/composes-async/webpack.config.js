const { CssExtractRspackPlugin } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [
					CssExtractRspackPlugin.loader,
					{
						loader: "css-loader",
						options: {
							modules: {
								localIdentName: "[local]"
							}
						}
					}
				]
			}
		]
	},
	optimization: {
		splitChunks: {
			minSize: 0,
			cacheGroups: {
				cssDedupe: {
					test: /\.css$/,
					name: "dedupe",
					chunks: "all",
					minChunks: 2
					// enforce: true
				}
			}
		}
	},
	plugins: [
		new CssExtractRspackPlugin({
			filename: "[name].css"
		})
	]
};
