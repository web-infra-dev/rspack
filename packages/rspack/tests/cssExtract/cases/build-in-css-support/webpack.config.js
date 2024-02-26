import { RspackCssExtractPlugin } from "../../../../src";

module.exports = {
	entry: "./index.js",
	output: {
		clean: false,
		cssFilename: "[name].css"
	},
	experiments: {
		css: true
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
