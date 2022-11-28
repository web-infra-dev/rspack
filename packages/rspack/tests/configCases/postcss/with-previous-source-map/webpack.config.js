const postcssLoader = require("@rspack/postcss-loader");
module.exports = {
	devtool: "source-map",
	module: {
		rules: [
			{
				test: /\.scss$/,
				use: [
					{
						loader: postcssLoader,
						options: {
							pxToRem: true
						}
					},
					{
						builtinLoader: "sass-loader"
					}
				],
				type: "css"
			}
		]
	}
};
