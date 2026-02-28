const rspack = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "web",
	entry: "./src/index.js",
	output: {
		filename: "[name].js"
	},
	experiments: {
		css: false
	},
	module: {
		rules: [
			{
				test: /\.css$/,
        type: 'javascript/auto',
				use: [rspack.CssExtractRspackPlugin.loader, "./css-loader"]
			}
		]
	},
	plugins: [new rspack.CssExtractRspackPlugin()],
	performance: false,
	optimization: {
		chunkIds: "named",
		usedExports: false,
		sideEffects: false,
		concatenateModules: false,
		moduleIds: "named",
		splitChunks: {
			chunks: "all",
			cacheGroups: {
				default: false,
				defaultVendors: false,
				fragment: {
					minChunks: 1,
					maxSize: 200 * 1024,

					// there are 2 css, each one of them are only 120 bytes which is less than minSize
					// so the total size of the css are 240 bytes which is greater than minSize
					// so the nodes are
					// [js js css js js js js js css]
					// if scan from left to right, the minSize can only satisfy when scan to the last css
					// if scan from right to left, the minSize can only satisfy when scan to the first css
					// so split chunks should remove problematic nodes, in this case the 2 css
					// and then recalculate the size of the rest of the nodes
					minSize: 200,
					priority: 10
				}
			}
		}
	}
};
