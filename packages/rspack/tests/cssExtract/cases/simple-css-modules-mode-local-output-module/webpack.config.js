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
								mode: "local",
								localIdentName: "foo__[name]__[local]"
							}
						}
					}
				]
			}
		]
	},
	output: {
		module: true
	},
	experiments: {
		outputModule: true
	},
	plugins: [
		new RspackCssExtractPlugin({
			filename: "[name].css"
		})
	]
};
