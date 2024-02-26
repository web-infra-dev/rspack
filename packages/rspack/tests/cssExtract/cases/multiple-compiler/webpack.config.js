import { RspackCssExtractPlugin } from "../../../../src";

module.exports = [
	{
		entry: "./index.js",
		output: {
			filename: "one-[name].js"
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
				filename: "one/[name].css"
			})
		]
	},
	{
		entry: "./index.js",
		output: {
			filename: "two-[name].js"
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
				filename: "two/[name].css"
			})
		]
	}
];
