const { rspack } = require("@rspack/core");

/**@type {import('@rspack/core').Configuration} */
module.exports = {
	entry: {
		polyfill: "./polyfill.js",
		main: "./index.js"
	},
	output: {
		filename: "[name].js"
	},
	plugins: [
		new rspack.HtmlRspackPlugin({
			template: "./index.html"
		})
	]
};
