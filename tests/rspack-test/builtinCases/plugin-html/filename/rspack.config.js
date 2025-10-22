const { HtmlRspackPlugin } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		new HtmlRspackPlugin({
			filename: "[name].[contenthash].html"
		}),
		new HtmlRspackPlugin({
			template: "./index.html",
			filename: "[name].[contenthash].html"
		})
	],
};
