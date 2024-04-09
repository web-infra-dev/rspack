const CssExtractRspackPlugin = require("../../../").default;

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
							publicPath: "https://webpack.js.org/foo/../"
						}
					},
					"css-loader"
				]
			}
		]
	},
	plugins: [
		new CssExtractRspackPlugin({
			filename: "[name].css"
		})
	],
	experiments: {
		css: false,
		rspackFuture: {
			newTreeshaking: true
		}
	}
};
