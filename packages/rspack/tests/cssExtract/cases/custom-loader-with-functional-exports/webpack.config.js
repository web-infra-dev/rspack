const path = require("path");
const { CssExtractRspackPlugin } = require("../../../../");

module.exports = {
	entry: "./index.js",
	context: path.resolve(__dirname, "app"),
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [CssExtractRspackPlugin.loader, "./mockLoader"]
			}
		]
	},
	plugins: [
		new CssExtractRspackPlugin({
			filename: "[name].css"
		})
	]
};
