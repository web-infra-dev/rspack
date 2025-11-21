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
			template: "index.html",
			templateParameters: {
				foo: "bar"
			}
		})
	],
};
