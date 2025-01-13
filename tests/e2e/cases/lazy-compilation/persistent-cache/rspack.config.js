const { rspack } = require("@rspack/core");

/** @type { import('@rspack/core').RspackOptions } */
module.exports = {
	context: __dirname,
	entry: {
		main: ["./src/component.js", "./src/index.js"]
	},
	stats: "none",
	mode: "production",
	plugins: [new rspack.HtmlRspackPlugin()],
	cache: true,
	experiments: {
		lazyCompilation: true,
		cache: {
			type: "persistent"
		}
	},
	devServer: {
		hot: true
	}
};
