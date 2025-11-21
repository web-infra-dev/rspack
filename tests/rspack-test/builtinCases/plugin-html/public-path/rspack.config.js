const { HtmlRspackPlugin } = require("@rspack/core");
/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		index: {
			import: ["./index.js"]
		}
	},
	output: {
		publicPath: "/base"
	},
	plugins: [
		new HtmlRspackPlugin({
			favicon: "favicon.ico"
		})
	],
};
