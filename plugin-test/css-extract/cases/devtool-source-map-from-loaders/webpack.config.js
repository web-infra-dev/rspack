const { CssExtractRspackPlugin } = require("@rspack/core");

module.exports = {
	entry: "./index.js",
	devtool: "source-map",
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [
					CssExtractRspackPlugin.loader,
					{
						loader: "css-loader",
						options: {
							sourceMap: true
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
