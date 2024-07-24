const path = require("path");

const { CssExtractRspackPlugin } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	context: path.resolve(__dirname, "app"),
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [
					{
						loader: CssExtractRspackPlugin.loader,
						options: {
							publicPath: "/public/"
						}
					},
					"./mockLoader"
				]
			},
			{
				test: /\.png$/,
				type: "asset/resource",
				generator: {
					filename: "[path][name][ext]"
				}
			}
		]
	},
	plugins: [
		new CssExtractRspackPlugin({
			filename: "[name].css"
		})
	]
};
