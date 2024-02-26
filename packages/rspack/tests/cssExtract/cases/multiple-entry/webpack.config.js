import { RspackCssExtractPlugin } from "../../../../src";

module.exports = {
	entry: {
		"main-one": "./index-one.js",
		"main-two": "./index-two.js"
	},
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [RspackCssExtractPlugin.loader, "css-loader"]
			}
		]
	},
	plugins: [
		new RspackCssExtractPlugin({
			filename: "[name].css"
		})
	]
};
