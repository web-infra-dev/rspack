const rspack = require("@rspack/core");
const { PerfseePlugin } = require("@perfsee/webpack");
/** @type {import('@rspack/cli').Configuration} */
const config = {
	context: __dirname,
	entry: {
		main: "./src/index.js"
	},
	plugins: [
		new PerfseePlugin({}),
		new rspack.HtmlRspackPlugin({
			template: "./index.html"
		})
	]
};
module.exports = config;
