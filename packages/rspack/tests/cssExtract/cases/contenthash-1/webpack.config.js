import { RspackCssExtractPlugin } from "../../../../src";

module.exports = {
	entry: "./index.js",
	mode: "production",
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
	plugins: [
		new RspackCssExtractPlugin({
			filename: "[name].[contenthash].css"
		})
	]
};
