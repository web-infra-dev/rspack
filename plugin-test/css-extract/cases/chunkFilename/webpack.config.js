const { CssExtractRspackPlugin } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [CssExtractRspackPlugin.loader, "css-loader"]
			}
		]
	},
	optimization: {
		chunkIds: "deterministic"
	},
	plugins: [
		new CssExtractRspackPlugin({
			filename: "[name].css",
			chunkFilename: "[id].[name].css"
		})
	]
};
