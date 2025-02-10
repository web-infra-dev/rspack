const { rspack } = require("@rspack/core");

/** @type {import("webpack").Configuration} */
module.exports = {
	plugins: [new rspack.CssExtractRspackPlugin({ ignoreOrder: true })],
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [rspack.CssExtractRspackPlugin.loader, "css-loader"]
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
	experiments: {
		css: false,
		incremental: true
	}
};
