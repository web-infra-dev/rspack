import { RspackCssExtractPlugin } from "../../../../";

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
					"css-loader"
				]
			},
			{
				test: /\.svg$/,
				type: "asset/resource",
				generator: {
					filename: "static/[name][ext][query]"
				}
			}
		]
	},
	plugins: [
		new RspackCssExtractPlugin({
			filename: "[name].css"
		})
	]
};
