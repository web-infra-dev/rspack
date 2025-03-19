const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		ab: "./ab.js",
		ba: "./ba.js"
	},
	output: {
		filename: "[name].js"
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
	plugins: [new rspack.CssExtractRspackPlugin({ ignoreOrder: false })],
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [rspack.CssExtractRspackPlugin.loader, "css-loader"],
				type: "javascript/auto"
			}
		]
	},
	experiments: {
		css: false
	}
};
