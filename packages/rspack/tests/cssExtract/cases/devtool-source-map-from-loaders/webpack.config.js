import { RspackCssExtractPlugin } from "../../../../src";

module.exports = {
	entry: "./index.js",
	devtool: "source-map",
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [
					RspackCssExtractPlugin.loader,
					{
						loader: "css-loader",
						options: {
							sourceMap: true
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
