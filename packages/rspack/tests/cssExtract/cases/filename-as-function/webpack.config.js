import { RspackCssExtractPlugin } from "../../../../src";

module.exports = {
	entry: {
		"demo/js/main": "./index.js"
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
		filename: "[name].js"
	},
	plugins: [
		new RspackCssExtractPlugin({
			filename: ({ chunk }) => `${chunk.name.replace("/js/", "/css/")}.css`
		})
	]
};
