const { CssExtractRspackPlugin } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	mode: "development",
	devtool: false,
	output: {
		hotUpdateChunkFilename: "[id].hot-update.js"
	},
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [
					{
						loader: CssExtractRspackPlugin.loader
					},
					"css-loader"
				]
			}
		]
	},
	experiments: {
		css: false
	},
	plugins: [
		new CssExtractRspackPlugin({
			filename: "[name].css"
		})
	]
};
