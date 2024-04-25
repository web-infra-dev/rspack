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
		filename: "[name].$[contenthash]$.js"
	},
	optimization: {
		chunkIds: "named"
	},
	plugins: [
		new CssExtractRspackPlugin({
			filename: "[name].$[contenthash]$.css"
		})
	]
};
