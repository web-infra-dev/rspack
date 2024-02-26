import { RspackCssExtractPlugin } from "../../../../src";

module.exports = {
	entry: "./index.js",
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [
					{
						loader: RspackCssExtractPlugin.loader,
						options: {
							publicPath: "auto"
						}
					},
					"css-loader"
				]
			},
			{
				test: /\.(svg|png)$/,
				type: "asset/resource",
				generator: { filename: "assets/[name][ext]" }
			}
		]
	},
	plugins: [
		new RspackCssExtractPlugin({
			filename: "styles/[name].css"
		})
	]
};
