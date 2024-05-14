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
	optimization: { chunkIds: "named" },
	plugins: [
		new CssExtractRspackPlugin({
			filename: "main.css"
		})
	]
};
