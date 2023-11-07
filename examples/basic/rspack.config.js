const rspack = require("@rspack/core");
const p = require('./p')
/** @type {import('@rspack/cli').Configuration} */
const config = {
	entry: "./src/index.js",
	plugins: [
		new rspack.HtmlRspackPlugin({
			template: "./index.html"
		}),
		new p.P({}),
	]
};
module.exports = config;
