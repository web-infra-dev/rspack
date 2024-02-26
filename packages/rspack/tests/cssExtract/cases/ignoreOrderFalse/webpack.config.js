import { RspackCssExtractPlugin } from "../../../../";

module.exports = {
	entry: {
		entry1: "./index.js",
		entry2: "./index2.js",
		entry3: "./index3.js"
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
			ignoreOrder: false
		})
	]
};
