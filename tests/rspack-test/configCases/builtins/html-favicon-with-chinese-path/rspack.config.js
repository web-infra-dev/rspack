const path = require("path");
const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		new rspack.HtmlRspackPlugin({
			publicPath: "/",
			favicon: "./资源/favicon-图标.ico"
		})
	]
};
