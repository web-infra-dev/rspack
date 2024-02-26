import { RspackCssExtractPlugin } from "../../../../src";

module.exports = {
	entry: {
		// Specific CSS entry point, with output to a nested folder
		"nested/style": "./nested/style.css",
		// Note that relative nesting of output is the same as that of the input
		"nested/again/style": "./nested/again/style.css"
	},
	output: {
		// Compute publicPath relative to the CSS output
		publicPath: pathData => `http://example.com/${pathData.hash}/`
	},
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
	plugins: [
		new RspackCssExtractPlugin({
			filename: "[name].css"
		})
	]
};
