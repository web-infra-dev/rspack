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
			filename: "output.html",
			template: "input.html",
			inject: "head",
			scriptLoading: "blocking",
		})
	],
};
