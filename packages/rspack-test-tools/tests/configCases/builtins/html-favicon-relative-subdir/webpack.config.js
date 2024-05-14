const path = require("path");
const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		new rspack.HtmlRspackPlugin({
			publicPath: "/",
			favicon: "./static/favicon.ico"
		})
	]
};
