import { RspackCssExtractPlugin } from "../../../../src";

module.exports = {
	entry: {
		entry1: "./entry1.js",
		entry2: "./entry2.js"
	},
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [RspackCssExtractPlugin.loader, "css-loader"]
			}
		]
	},
	optimization: {
		splitChunks: {
			cacheGroups: {
				styles: {
					name: "styles",
					chunks: "all",
					test: /\.css$/,
					enforce: true
				}
			}
		}
	},
	plugins: [
		new RspackCssExtractPlugin({
			filename: "[name].css"
		})
	]
};
