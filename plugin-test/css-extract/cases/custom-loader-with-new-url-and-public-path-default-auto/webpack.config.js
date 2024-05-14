const path = require("path");

const { CssExtractRspackPlugin } = require("@rspack/core");

module.exports = {
	entry: "./index.js",
	context: path.resolve(__dirname, "app"),
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [CssExtractRspackPlugin.loader, "./mockLoader"]
			},
			{
				test: /\.png$/,
				type: "asset/resource",
				generator: {
					filename: "[path][name][ext]"
				}
			}
		]
	},
	plugins: [
		new CssExtractRspackPlugin({
			filename: "[name].css"
		})
	]
};
