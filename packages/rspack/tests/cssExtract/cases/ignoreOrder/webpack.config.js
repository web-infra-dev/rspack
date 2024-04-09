const { CssExtractRspackPlugin } = require("../../../../");

module.exports = {
	entry: {
		entry1: "./index.js",
		entry2: "./index2.js"
	},
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [CssExtractRspackPlugin.loader, "css-loader"]
			}
		]
	},
	optimization: {
		splitChunks: {
			cacheGroups: {
				styles: {
					name: "styles",
					chunks: "all",
					test: /\.css$/,
					enforce: true
				}
			}
		}
	},
	plugins: [
		new CssExtractRspackPlugin({
			ignoreOrder: true
		})
	]
};
