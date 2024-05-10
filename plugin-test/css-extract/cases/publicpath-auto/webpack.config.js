const { CssExtractRspackPlugin } = require("@rspack/core");

module.exports = {
	entry: "./index.js",
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [
					{
						loader: CssExtractRspackPlugin.loader,
						options: {
							publicPath: "auto"
						}
					},
					"css-loader"
				]
			},
			{
				test: /\.(svg|png)$/,
				type: "asset/resource",
				generator: { filename: "assets/[name][ext]" }
			}
		]
	},
	plugins: [
		new CssExtractRspackPlugin({
			filename: "styles/[name].css"
		})
	]
};
