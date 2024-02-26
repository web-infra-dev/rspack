import { RspackCssExtractPlugin } from "../../../../src";

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
				use: [RspackCssExtractPlugin.loader, "css-loader"]
			}
		]
	},
	output: {
		filename: "[name]-[contenthash].js"
	},
	plugins: [
		new RspackCssExtractPlugin({
			filename: "[name]-[contenthash].css"
		})
	]
};
