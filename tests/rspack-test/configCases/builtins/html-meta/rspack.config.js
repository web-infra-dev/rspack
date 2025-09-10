const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
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
