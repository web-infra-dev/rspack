import { RspackCssExtractPlugin } from "../../../../src";

module.exports = {
	entry: "./index.js",
	module: {
		rules: [
			{
				oneOf: [
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
						exclude: /\.(js|mjs|jsx|ts|tsx)$/,
						type: "asset/resource",
						generator: {
							filename: "[name][ext]"
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
