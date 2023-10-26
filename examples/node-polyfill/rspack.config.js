const rspack = require("@rspack/core");
const polyfillPlugin = require("@rspack/plugin-node-polyfill");
/** @type {import('@rspack/cli').Configuration} */
const config = {
	context: __dirname,
	entry: {
		main: "./src/index.js"
	},
	plugins: [
		new polyfillPlugin(),
		new rspack.HtmlRspackPlugin({
			template: "./index.html"
		})
	]
};
module.exports = config;
