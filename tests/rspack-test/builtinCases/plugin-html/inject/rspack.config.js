const { HtmlRspackPlugin } = require("@rspack/core");
/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		new HtmlRspackPlugin({
			filename: "inject_head.html",
			inject: "head"
		}),
		new HtmlRspackPlugin({
			filename: "inject_body.html",
			inject: "body"
		}),
		new HtmlRspackPlugin({
			filename: "inject_false.html",
			inject: false
		})
	],
};
