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
			title: "Rspack title"
		})
	],
};
