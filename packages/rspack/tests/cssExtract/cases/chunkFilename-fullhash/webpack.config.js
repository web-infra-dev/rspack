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
		chunkIds: "named"
	},
	plugins: [
		new RspackCssExtractPlugin({
			filename: "[fullhash].css",
			chunkFilename: "[id].[fullhash].css"
		})
	]
};
