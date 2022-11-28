const postcssLoader = require("@rspack/postcss-loader");
module.exports = {
	devtool: "source-map",
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [
					{
						loader: postcssLoader,
						options: {
							pxToRem: true
						}
					}
				]
			}
		]
	}
};
