const { HtmlRspackPlugin } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		index: {
			import: ["./index.js"]
		}
	},
	plugins: [
		new HtmlRspackPlugin({
			templateContent:
				"<!DOCTYPE html><html><body><div><%= foo %></div></body></html>",
			templateParameters: {
				foo: "bar"
			}
		})
	],
};
