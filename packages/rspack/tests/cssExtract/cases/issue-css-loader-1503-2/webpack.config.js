import { RspackCssExtractPlugin } from "../../../../src";

module.exports = {
	entry: {
		dark: "./dark.css",
		index: "./index.css"
	},
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
							modules: true
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
