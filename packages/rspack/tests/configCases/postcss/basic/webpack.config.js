const postcssLoader = require("@rspack/plugin-postcss");
module.exports = {
	module: {
		rules: [
			{
				test: /\.module\.css$/,
				uses: [
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
				uses: [
					{
						loader: postcssLoader
					}
				]
			}
		]
	}
};
