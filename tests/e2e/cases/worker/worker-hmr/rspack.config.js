const rspack = require("@rspack/core");

/** @type {rspack.Configuration} */
module.exports = {
	context: __dirname,
	entry: {
		main: "./src/index.js"
	},
	devtool: false,
	mode: "development",
	plugins: [new rspack.HtmlRspackPlugin({ template: "./src/index.html" })],
	devServer: {
		hot: true
	}
};
