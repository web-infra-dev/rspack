const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		publicPath: "/base"
	},
	plugins: [new rspack.HtmlRspackPlugin({})]
};
