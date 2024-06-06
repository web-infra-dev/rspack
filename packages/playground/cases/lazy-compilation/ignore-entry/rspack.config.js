const rspack = require("@rspack/core");

/** @type { import('@rspack/core').RspackOptions } */
module.exports = {
	context: __dirname,
	entry: {
		main: [
			// Will trigger the issue.
			'data:text/javascript,import "core-js";',
			"./src/index.css",
			"./src/index.js"
		]
	},
	stats: "none",
	mode: "development",
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [rspack.CssExtractRspackPlugin.loader, "css-loader"]
			}
		]
	},
	plugins: [new rspack.HtmlRspackPlugin(), new rspack.CssExtractRspackPlugin()],
	experiments: {
		css: false,
		lazyCompilation: true
	},
	devServer: {
		hot: true
	}
};
