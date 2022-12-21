const postcssLoader = require("@rspack/postcss-loader");
module.exports = {
	builtins: {
		treeShaking: true,
		sideEffects: true
	},
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
