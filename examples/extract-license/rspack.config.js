const rspack = require("@rspack/core");
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
		new rspack.SwcJsMinimizerRspackPlugin({
			extractComments: true
		})
	]
};
module.exports = config;
