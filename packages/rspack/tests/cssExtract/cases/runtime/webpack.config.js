import { RspackCssExtractPlugin } from "../../../../src";

module.exports = {
	entry: "./index.js",
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [RspackCssExtractPlugin.loader, "css-loader"]
			}
		]
	},
	output: {
		filename: `[name].js`
	},
	plugins: [
		new RspackCssExtractPlugin({
			filename: `[name].css`,
			chunkFilename: `[name].css`
		})
	],
	optimization: {
		runtimeChunk: true
	}
};
