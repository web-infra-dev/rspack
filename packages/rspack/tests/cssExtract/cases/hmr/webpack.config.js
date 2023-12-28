const {
	CssExtractRspackPlugin,
	HotModuleReplacementPlugin
} = require("../../../../");

module.exports = {
	entry: "./index.css",
	mode: "development",
	devtool: false,
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [
					{
						loader: CssExtractRspackPlugin.loader
					},
					"css-loader"
				]
			}
		]
	},
	devServer: { hot: true },
	plugins: [
		new HotModuleReplacementPlugin(),
		new CssExtractRspackPlugin({
			filename: "[name].css"
		})
	]
};
