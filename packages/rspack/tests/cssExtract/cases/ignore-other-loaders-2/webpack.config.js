const { CssExtractRspackPlugin } = require("../../../../");

module.exports = {
	entry: "./index.js",
	module: {
		rules: [
			{
				oneOf: [
					{
						test: /\.css$/,
						use: [
							{
								loader: CssExtractRspackPlugin.loader
							},
							"css-loader"
						]
					},
					{
						exclude: /\.(js|mjs|jsx|ts|tsx)$/,
						type: "asset/resource",
						generator: {
							filename: "[name][ext]"
						}
					}
				]
			}
		]
	},
	plugins: [
		new CssExtractRspackPlugin({
			filename: "[name].css"
		})
	]
};
