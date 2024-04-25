const { rspack } = require("@rspack/core");

module.exports = {
	plugins: [
		new rspack.HtmlRspackPlugin({
			templateContent:
				"<!DOCTYPE html><html><body><div><%= env %></div></body></html>",
			templateParameters: {
				env: "production"
			}
		})
	]
};
