const { rspack } = require("@rspack/core");

module.exports = {
	plugins: [
		new rspack.HtmlRspackPlugin({
			meta: {
				viewport: "width=device-width, initial-scale=1, shrink-to-fit=no",
				test: {
					a: "b"
				}
			}
		})
	]
};
