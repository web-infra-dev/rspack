const { CssExtractRspackPlugin } = require("../../../../");

module.exports = {
	entry: "./index.js",
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [
					CssExtractRspackPlugin.loader,
					{
						loader: "css-loader",
						options: {
							modules: {
								mode: "global",
								localIdentName: "foo__[name]__[local]"
							}
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
