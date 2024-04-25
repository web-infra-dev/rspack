const { CssExtractRspackPlugin } = require("@rspack/core");

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
