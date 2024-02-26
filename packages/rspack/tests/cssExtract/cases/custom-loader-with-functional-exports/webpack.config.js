import path from "path";

import { RspackCssExtractPlugin } from "../../../../src";

module.exports = {
	entry: "./index.js",
	context: path.resolve(__dirname, "app"),
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [RspackCssExtractPlugin.loader, "./mockLoader"]
			}
		]
	},
	plugins: [
		new RspackCssExtractPlugin({
			filename: "[name].css"
		})
	]
};
