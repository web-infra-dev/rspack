const rspack = require("@rspack/core");

/** @type { import('@rspack/core').RspackOptions } */
module.exports = {
	context: __dirname,
	entry: "./src/index.js",
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
