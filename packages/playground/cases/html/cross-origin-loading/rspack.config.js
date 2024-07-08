const { rspack } = require("@rspack/core");

/** @type { import('@rspack/core').RspackOptions } */
module.exports = {
	context: __dirname,
	mode: "development",
	entry: "./src/index.js",
	stats: "none",
	plugins: [
		new rspack.HtmlRspackPlugin({
			template: "./src/index.html"
		})
	],
	output: {
		crossOriginLoading: "anonymous"
	},
	devServer: {
		port: 3000
	}
};
