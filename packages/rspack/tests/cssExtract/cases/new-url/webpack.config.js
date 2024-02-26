import { RspackCssExtractPlugin } from "../../../../src";

module.exports = {
	entry: "./index.js",
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [RspackCssExtractPlugin.loader, "css-loader"],
				type: "javascript/auto"
			}
		]
	},
	plugins: [
		new RspackCssExtractPlugin({
			filename: "[name].css"
		})
	]
};
