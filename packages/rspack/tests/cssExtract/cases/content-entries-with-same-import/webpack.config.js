import { RspackCssExtractPlugin } from "../../../../src";

module.exports = {
	entry: "./index.js",
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [RspackCssExtractPlugin.loader, "css-loader"]
			}
		]
	},
	output: {
		filename: "[name].[contenthash].js"
	},
	optimization: {
		chunkIds: "named"
	},
	plugins: [
		new RspackCssExtractPlugin({
			filename: "[name].[contenthash].css"
		})
	]
};
