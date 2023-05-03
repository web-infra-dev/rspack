const path = require("path");
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
		],
		provide: {
			process: path.resolve(__dirname, "./src/process-shim.js")
		}
	}
};
module.exports = config;
