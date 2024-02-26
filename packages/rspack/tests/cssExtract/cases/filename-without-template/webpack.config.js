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
	optimization: { chunkIds: "named" },
	plugins: [
		new RspackCssExtractPlugin({
			filename: "main.css"
		})
	]
};
