const { CssExtractRspackPlugin } = require("../../../../");

module.exports = {
	entry: "./index.js",
	// Required to disable source maps in webpack@4
	devtool: false,
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
							sourceMap: false
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
