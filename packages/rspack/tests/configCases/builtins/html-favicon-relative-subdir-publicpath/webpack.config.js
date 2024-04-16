const path = require("path");
const { rspack } = require("@rspack/core");

module.exports = {
	plugins: [
		new rspack.HtmlRspackPlugin({
			publicPath: "/assets/",
			favicon: "./static/favicon.ico"
		})
	]
};
