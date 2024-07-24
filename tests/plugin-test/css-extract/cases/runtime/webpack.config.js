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
	output: {
		filename: `[name].js`
	},
	plugins: [
		new CssExtractRspackPlugin({
			filename: `[name].css`,
			chunkFilename: `[name].css`
		})
	],
	optimization: {
		runtimeChunk: true
	}
};
