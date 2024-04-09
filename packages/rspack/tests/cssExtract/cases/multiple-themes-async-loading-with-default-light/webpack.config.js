const { CssExtractRspackPlugin } = require("../../../../");

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
							CssExtractRspackPlugin.loader,
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
							CssExtractRspackPlugin.loader,
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
		new CssExtractRspackPlugin({
			filename: "[name].css",
			attributes: {
				id: "theme"
			}
		})
	]
};
