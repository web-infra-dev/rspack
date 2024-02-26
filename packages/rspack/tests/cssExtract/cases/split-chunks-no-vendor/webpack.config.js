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
		splitChunks: {
			chunks: "all",
			cacheGroups: {
				default: false,
				vendors: false,
				vendor: {
					test: /[\\/]node_modules[\\/]/,
					name: "vendor",
					chunks: "all",
					enforce: true
				},
				styles: {
					name: "bundle",
					type: "css/mini-extract",
					chunks: "all",
					priority: 100,
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
