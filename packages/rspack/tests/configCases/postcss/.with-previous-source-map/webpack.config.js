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
					{ loader: "builtin:sass-loader" }
				],
				type: "css"
			}
		]
	}
};
