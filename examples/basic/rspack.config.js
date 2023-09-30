const rspack = require("@rspack/core");
/** @type {import('@rspack/cli').Configuration} */
const config = {
	context: __dirname,
	mode: "development",
	entry: {
		main: "./src/index.js"
	},
	builtins: {
		html: [
			{
				template: "./index.html"
			}
		]
	},
	plugins: [
		new rspack.FooterRspackPlugin({
			footer: "\n/* js footer */\n"
		})
	]
};
module.exports = config;
