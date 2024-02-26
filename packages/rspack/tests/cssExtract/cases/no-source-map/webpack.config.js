import { RspackCssExtractPlugin } from "../../../../src";

module.exports = {
	entry: "./index.js",
	// Required to disable source maps in webpack@4
	devtool: false,
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
							sourceMap: false
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
