const rspack = require("@rspack/core");

/** @type { import('@rspack/core').RspackOptions } */
module.exports = {
	context: __dirname,
	entry: {
		main: [
			// Will trigger the issue.
			'data:text/javascript,import "core-js";',
			"./src/index.js"
		]
	},
	stats: "none",
	mode: "development",
	plugins: [new rspack.HtmlRspackPlugin()],
	experiments: {
		lazyCompilation: {
			entries: true
		}
	},
	devServer: {
		hot: true
	}
};
