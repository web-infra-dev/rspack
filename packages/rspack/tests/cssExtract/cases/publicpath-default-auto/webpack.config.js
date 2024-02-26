import { RspackCssExtractPlugin } from "../../../../";

module.exports = {
	entry: "./index.js",
	output: {
		publicPath: "auto"
	},
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [RspackCssExtractPlugin.loader, "css-loader"]
			}
		]
	},
	plugins: [
		new RspackCssExtractPlugin({
			filename: "[name].css"
		})
	]
};
