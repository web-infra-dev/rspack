const postcssLoader = require("@rspack/plugin-postcss");
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
