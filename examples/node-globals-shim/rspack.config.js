const rspack = require("@rspack/core");
const path = require("path");
/** @type {import('@rspack/cli').Configuration} */
const config = {
	context: __dirname,
	entry: {
		main: "./src/index.js"
	},
	plugins: [
		new rspack.HtmlRspackPlugin({
			template: "./index.html"
		}),
		new rspack.ProvidePlugin({
			process: path.resolve(__dirname, "./src/process-shim.js")
		})
	]
};
module.exports = config;
