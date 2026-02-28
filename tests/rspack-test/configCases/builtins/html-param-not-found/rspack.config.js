const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		new rspack.HtmlRspackPlugin({
			templateContent:
				"<!DOCTYPE html><html><head><title><%= title %></title></head><body></body></html>"
		})
	]
};
