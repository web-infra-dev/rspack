const path = require("path");
const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		new rspack.HtmlRspackPlugin({
			publicPath: "/assets/",
			favicon: "./static/favicon.ico"
		})
	]
};
