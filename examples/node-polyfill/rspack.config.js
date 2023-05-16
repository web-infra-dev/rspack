const polyfillPlugin = require("@rspack/plugin-node-polyfill");
/** @type {import('@rspack/cli').Configuration} */
const config = {
	context: __dirname,
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
	plugins: [new polyfillPlugin()]
};
module.exports = config;
