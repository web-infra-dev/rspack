const path = require("path");
const HtmlPlugin = require("html-webpack-plugin");
const rspack = require("@rspack/core");

module.exports = {
	plugins: [
		new HtmlPlugin({
			template: "./document.ejs"
		}),
		new rspack.DefinePlugin({
			title: JSON.stringify("CUSTOM TITLE")
		})
	]
};
