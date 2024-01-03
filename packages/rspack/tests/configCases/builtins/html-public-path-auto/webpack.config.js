const path = require("path");
const rspack = require("@rspack/core");

module.exports = {
	target: "web",
	externals: {
		path: "require('path')",
		fs: "require('fs')"
	},
	plugins: [
		new rspack.DefinePlugin({
			__dirname: JSON.stringify(path.join(__dirname, "./dist"))
		}),
		new rspack.HtmlRspackPlugin({
			filename: "main_page/index.html"
		})
	]
};
