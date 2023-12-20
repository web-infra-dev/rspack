const rspack = require("@rspack/core");
/** @type {import('@rspack/cli').Configuration} */
const config = {
	entry: "./src/index.js",
	plugins: [
		new rspack.HtmlRspackPlugin({
			template: "./index.html"
		})
	],
	bail: true
};
module.exports = config;
