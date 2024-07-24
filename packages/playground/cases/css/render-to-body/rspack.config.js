const { rspack } = require("@rspack/core");

/** @type { import('@rspack/core').RspackOptions } */
module.exports = {
	context: __dirname,
	mode: "development",
	entry: "./src/index.js",
	devServer: {
		hot: true
	},
	stats: "none",
	infrastructureLogging: {
		debug: false
	},
	plugins: [
		new rspack.HtmlRspackPlugin({
			template: "./src/index.html",
			inject: "body"
		})
	],
	watchOptions: {
		poll: 1000
	},
	experiments: {
		css: true
	}
};
