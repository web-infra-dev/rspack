const postcssLoader = require("@rspack/plugin-postcss");
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
