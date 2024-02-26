import { RspackCssExtractPlugin } from "../../../../src";

module.exports = {
	entry: {
		a: "./a.js",
		b: "./b.js"
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
					type: "css/mini-extract",
					chunks: "all",
					enforce: true
				}
			}
		}
	},
	plugins: [new RspackCssExtractPlugin()]
};
