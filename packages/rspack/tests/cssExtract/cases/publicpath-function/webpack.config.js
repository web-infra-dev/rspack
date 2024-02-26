import path from "path";

import { RspackCssExtractPlugin } from "../../../../src";

module.exports = {
	entry: {
		// Specific CSS entry point, with output to a nested folder
		"nested/style": "./nested/style.css",
		// Note that relative nesting of output is the same as that of the input
		"nested/again/style": "./nested/again/style.css"
	},
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [
					{
						loader: RspackCssExtractPlugin.loader,
						options: {
							// Compute publicPath relative to the CSS output
							publicPath: (resourcePath, context) =>
								`${path
									.relative(path.dirname(resourcePath), context)
									.replace(/\\/g, "/")}/`
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
	]
};
