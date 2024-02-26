import { RspackCssExtractPlugin } from "../../../../src";

module.exports = {
	entry: "./index.js",
	devtool: "source-map",
	output: {
		// pathinfo: true
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
			filename: "[name].css",
			pathinfo: true
		})
	]
};
