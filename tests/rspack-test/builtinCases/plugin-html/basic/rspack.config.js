const { HtmlRspackPlugin } = require("@rspack/core");
/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		new HtmlRspackPlugin({})
	],
};
