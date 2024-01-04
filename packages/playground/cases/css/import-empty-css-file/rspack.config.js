const { rspack } = require("@rspack/core");

/** @type { import('@rspack/core').RspackOptions } */
module.exports = {
	context: __dirname,
	mode: "development",
	entry: "./src/index.js",
	devServer: {
		hot: true
	},
	cache: false,
	stats: "none",
	infrastructureLogging: {
		debug: false
	},
	plugins: [
		new rspack.HtmlRspackPlugin({
			template: "./src/index.html"
		})
	],
	watchOptions: {
		poll: 1000
	}
};
