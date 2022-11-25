const postcssLoader = require("@rspack/plugin-postcss");
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
