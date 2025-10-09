const { CssExtractRspackPlugin } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [CssExtractRspackPlugin.loader, "css-loader"],
				type: "javascript/auto"
			},
			{
				resourceQuery: /layer1/,
				layer: "layer1"
			},
			{
				resourceQuery: /layer2/,
				layer: "layer2"
			}
		]
	},
	optimization: {
		splitChunks: {
			chunks: "all",
			cacheGroups: {
				default: false,
				layer1: {
					reuseExistingChunk: false,
					layer: "layer1",
					name: "layer1",
					enforce: true
				},
				layer2: {
					reuseExistingChunk: false,
					layer: "layer2",
					name: "layer2",
					enforce: true
				}
			}
		}
	},
	plugins: [
		new CssExtractRspackPlugin({
			chunkFilename: "[name].css"
		})
	],
	experiments: {
		css: false
	}
};
