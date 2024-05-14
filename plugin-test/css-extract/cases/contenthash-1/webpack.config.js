const { CssExtractRspackPlugin } = require("@rspack/core");

module.exports = {
	entry: "./index.js",
	mode: "production",
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
	plugins: [
		new CssExtractRspackPlugin({
			filename: "[name].$[contenthash]$.css"
		})
	]
};
