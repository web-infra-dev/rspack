import { RspackCssExtractPlugin } from "../../../../src";

module.exports = {
	entry: "./index.js",
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [
					RspackCssExtractPlugin.loader,
					{
						loader: "css-loader",
						options: {
							modules: {
								mode: "global",
								localIdentName: "foo__[name]__[local]"
							}
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
