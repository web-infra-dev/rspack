import { RspackCssExtractPlugin } from "../../../../src";

module.exports = [1, 2].map(n => {
	return {
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
			filename: `${n}.[name].js`
		},
		resolve: {
			alias: {
				"./style.css": `./style${n}.css`
			}
		},
		plugins: [
			new RspackCssExtractPlugin({
				filename: `${n}.[name].[contenthash].css`
			})
		]
	};
});
