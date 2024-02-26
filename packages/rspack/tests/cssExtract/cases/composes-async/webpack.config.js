import { RspackCssExtractPlugin } from "../../../../src";

module.exports = {
	entry: "./index.js",
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [
					RspackCssExtractPlugin.loader,
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
		new RspackCssExtractPlugin({
			filename: "[name].css"
		})
	]
};
