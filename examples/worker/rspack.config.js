const rspack = require("@rspack/core");
const path = require("path");

module.exports = {
	entry: "./example.js",
	context: __dirname,
	output: {
		path: path.join(__dirname, "dist")
	},
	plugins: [
		new rspack.HtmlRspackPlugin({
			template: "./index.html"
		})
	]
};
