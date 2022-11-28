const postcssLoader = require("@rspack/postcss-loader");
module.exports = {
	module: {
		rules: [
			{
				test: /\.module\.css$/,
				use: [
					{
						loader: postcssLoader,
						options: {
							modules: true
						}
					}
				]
			},
			{
				test: /\.css$/,
				use: [
					{
						loader: postcssLoader
					}
				]
			}
		]
	}
};
