const { CssExtractRspackPlugin } = require("@rspack/core");

module.exports = {
	entry: "./index.js",
	output: {
		clean: false,
		cssFilename: "[name].css"
	},
	experiments: {
		css: true
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
