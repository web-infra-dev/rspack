const { CssExtractRspackPlugin } = require("../../../../");

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
		chunkIds: "named"
	},
	plugins: [
		new CssExtractRspackPlugin({
			filename: "[name].$[fullhash]$.css",
			chunkFilename: "[id].$[fullhash]$.css"
		})
	]
};
