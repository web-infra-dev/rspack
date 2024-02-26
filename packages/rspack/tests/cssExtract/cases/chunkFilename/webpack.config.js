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
	optimization: {
		chunkIds: "deterministic"
	},
	plugins: [
		new RspackCssExtractPlugin({
			filename: "[name].css",
			chunkFilename: "[id].[name].css"
		})
	]
};
