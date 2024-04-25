const { CssExtractRspackPlugin } = require("@rspack/core");

module.exports = {
	entry: "./index.js",
	output: {
		publicPath: "auto"
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
