const { CssExtractRspackPlugin } = require("../../../../");

module.exports = {
	entry: "./index.js",
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [
					{
						loader: CssExtractRspackPlugin.loader
					},
					{
						loader: "css-loader",
						options: {
							esModule: false
						}
					}
				]
			},
			{
				test: /\.svg$/,
				type: "javascript/auto",
				use: [
					{
						loader: "file-loader",
						options: {
							name: "static/[name].[ext]"
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
