import { RspackCssExtractPlugin } from "../../../../src";

module.exports = {
	entry: "./index.js",
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [
					RspackCssExtractPlugin.loader,
					{
						loader: "css-loader"
						// TODO Uncomment after `css-loader` release the `esModule` option
						// options: { esModule: false },
					}
				]
			}
		]
	},
	plugins: [
		new RspackCssExtractPlugin({
			filename: "[name].css"
		})
	]
};
