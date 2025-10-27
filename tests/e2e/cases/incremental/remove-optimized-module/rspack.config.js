const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	cache: true,
	experiments: {
		cache: true,
		incremental: {
			providedExports: false
		}
	},
	plugins: [new rspack.HtmlRspackPlugin()]
};
