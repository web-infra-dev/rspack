const path = require("path");
const { rspack } = require("@rspack/core");

module.exports = {
	plugins: [
		new rspack.HtmlRspackPlugin({
			publicPath: "/",
			favicon: "favicon.ico"
		})
	]
};
