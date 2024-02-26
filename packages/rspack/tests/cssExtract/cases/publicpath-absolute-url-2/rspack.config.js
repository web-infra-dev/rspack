const RspackCssExtractPlugin = require("../../../").default;

module.exports = {
	entry: "./index.js",
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [
					{
						loader: RspackCssExtractPlugin.loader,
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
		new RspackCssExtractPlugin({
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
