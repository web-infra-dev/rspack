import { RspackCssExtractPlugin } from "../../../../src";

module.exports = {
	entry: "./src/index.js",
	module: {
		rules: [
			{
				test: /\.s[ac]ss$/i,
				oneOf: [
					{
						resourceQuery: "?dark",
						use: [
							RspackCssExtractPlugin.loader,
							"css-loader",
							{
								loader: "sass-loader",
								options: {
									additionalData: `@use 'dark-theme/vars' as vars;`
								}
							}
						]
					},
					{
						use: [
							RspackCssExtractPlugin.loader,
							"css-loader",
							{
								loader: "sass-loader",
								options: {
									additionalData: `@use 'light-theme/vars' as vars;`
								}
							}
						]
					}
				]
			}
		]
	},
	plugins: [
		new RspackCssExtractPlugin({
			filename: "[name].css",
			attributes: {
				id: "theme"
			}
		})
	]
};
