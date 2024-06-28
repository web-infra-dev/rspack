const { CssExtractRspackPlugin } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
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
						loader: CssExtractRspackPlugin.loader,
						options: {
							esModule: true
						}
					},
					"css-loader"
				]
			}
		]
	},
	plugins: [
		new CssExtractRspackPlugin({
			filename: "[name].css"
		})
	]
};
