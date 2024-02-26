import { RspackCssExtractPlugin } from "../../../../src";

module.exports = {
	mode: "development",
	entry: ["./a.css", "./b.css"],
	output: {
		pathinfo: true
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
