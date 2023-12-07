const rspack = require("@rspack/core");
/** @type {import('@rspack/cli').Configuration} */
const config = {
	entry: "./src/index.js",
	plugins: [
		new rspack.HtmlRspackPlugin({
			template: "./index.html"
		})
	],
	builtins: {
		noEmitAssets: true
	}
};
module.exports = config;
