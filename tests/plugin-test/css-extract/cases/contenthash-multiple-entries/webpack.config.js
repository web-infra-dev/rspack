const { CssExtractRspackPlugin } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		entryA: "./entryA.js",
		entryB: "./entryB.js",
		entryC: "./entryC.js",
		entryD: "./entryD.js",
		entryE: "./entryE.js"
	},
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [CssExtractRspackPlugin.loader, "css-loader"]
			}
		]
	},
	output: {
		filename: "[name].$[contenthash]$.js"
	},
	plugins: [
		new CssExtractRspackPlugin({
			filename: "[name].$[contenthash]$.css"
		})
	]
};
