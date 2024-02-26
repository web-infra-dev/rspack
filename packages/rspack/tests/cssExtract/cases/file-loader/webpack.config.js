import { RspackCssExtractPlugin } from "../../../../src";

module.exports = {
	entry: "./index.js",
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [
					{
						loader: RspackCssExtractPlugin.loader
					},
					{
						loader: "css-loader",
						options: {
							esModule: false
						}
					}
				]
			},
			{
				test: /\.svg$/,
				type: "javascript/auto",
				use: [
					{
						loader: "file-loader",
						options: {
							name: "static/[name].[ext]"
						}
					}
				]
			}
		]
	},
	plugins: [
		new RspackCssExtractPlugin({
			filename: "[name].css"
		})
	]
};
