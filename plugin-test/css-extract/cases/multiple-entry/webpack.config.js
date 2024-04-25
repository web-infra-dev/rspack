const { CssExtractRspackPlugin } = require("@rspack/core");

module.exports = {
	entry: {
		"main-one": "./index-one.js",
		"main-two": "./index-two.js"
	},
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [CssExtractRspackPlugin.loader, "css-loader"]
			}
		]
	},
	plugins: [
		new CssExtractRspackPlugin({
			filename: "[name].css"
		})
	]
};
