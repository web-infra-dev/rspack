import {
	RspackCssExtractPlugin,
	HotModuleReplacementPlugin
} from "../../../../";

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
						loader: RspackCssExtractPlugin.loader
					},
					"css-loader"
				]
			}
		]
	},
	devServer: { hot: true },
	plugins: [
		new HotModuleReplacementPlugin(),
		new RspackCssExtractPlugin({
			filename: "[name].css"
		})
	]
};
