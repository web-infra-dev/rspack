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
	lazyCompilation: true,
	experiments: {
		cache: {
			type: "persistent"
		}
	},
	devServer: {
		hot: true,
		client: {
			overlay: {
				// hide warnings for incremental
				warnings: false,
			}
		}
	}
};
