const { CssExtractRspackPlugin } = require("../../../../");

module.exports = {
	entry: "./index.js",
	optimization: {
		concatenateModules: true
	},
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [
					{
						loader: CssExtractRspackPlugin.loader,
						options: {
							esModule: true
						}
					},
					{
						loader: "css-loader",
						options: {
							esModule: true,
							modules: {
								namedExport: true,
								localIdentName: "foo__[local]"
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
