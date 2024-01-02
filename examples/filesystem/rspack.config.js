const rspack = require("@rspack/core");
/** @type {import('@rspack/cli').Configuration} */
const config = {
	entry: "./src/index.js",
	output: { clean: true },
	plugins: [
		new rspack.HtmlRspackPlugin({
			template: "./index.html"
		})
	],
	optimization: {
		minimize: false
	}
};
module.exports = config;
