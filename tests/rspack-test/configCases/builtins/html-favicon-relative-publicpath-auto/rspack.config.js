const path = require("path");
const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		publicPath: "auto"
	},
	plugins: [
		new rspack.HtmlRspackPlugin({
			favicon: "favicon.ico"
		})
	]
};
