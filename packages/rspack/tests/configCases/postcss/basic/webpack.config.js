const postcssLoader = require("@rspack/postcss-loader");
module.exports = {
	module: {
		defaultRules: [],
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
				],
				type: "css"
			},
			{
				test: /\.css$/,
				use: [
					{
						loader: postcssLoader
					}
				],
				type: "css"
			}
		]
	}
};
