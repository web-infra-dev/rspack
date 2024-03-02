const path = require("path");
const rspack = require("@rspack/core");

module.exports = {
	target: "web",
	externals: {
		path: "require('path')",
		fs: "require('fs')"
	},
	node: {
		__dirname: false
	},
	plugins: [
		new rspack.HtmlRspackPlugin({
			filename: "main_page/index.html"
		})
	]
};
