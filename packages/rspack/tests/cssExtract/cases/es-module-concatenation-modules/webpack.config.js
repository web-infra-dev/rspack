import { RspackCssExtractPlugin } from "../../../../src";

module.exports = {
	entry: "./index.js",
	optimization: {
		concatenateModules: true
	},
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [
					{
						loader: RspackCssExtractPlugin.loader,
						options: {
							esModule: true
						}
					},
					{
						loader: "css-loader",
						options: {
							esModule: true,
							modules: {
								namedExport: true,
								localIdentName: "foo__[local]"
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
