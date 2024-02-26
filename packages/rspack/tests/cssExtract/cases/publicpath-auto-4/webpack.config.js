import path from "path";

import { RspackCssExtractPlugin } from "../../../../src";

module.exports = {
	entry: "./index.js",
	context: path.resolve(__dirname, "app"),
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [
					{
						loader: RspackCssExtractPlugin.loader,
						options: {
							publicPath: "auto"
						}
					},
					"css-loader"
				]
			},
			{
				test: /\.png$/,
				type: "asset/resource",
				generator: {
					filename: "[path][name][ext]"
				}
			}
		]
	},
	plugins: [
		new RspackCssExtractPlugin({
			filename: "[name].css"
		})
	]
};
