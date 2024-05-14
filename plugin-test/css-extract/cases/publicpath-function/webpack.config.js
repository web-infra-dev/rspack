const path = require("path");

const { CssExtractRspackPlugin } = require("@rspack/core");

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
						loader: CssExtractRspackPlugin.loader,
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
		new CssExtractRspackPlugin({
			filename: "[name].css"
		})
	]
};
