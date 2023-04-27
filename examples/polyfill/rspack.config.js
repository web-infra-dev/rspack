const path = require("path");
/** @type {import('@rspack/cli').Configuration} */
const config = {
	context: __dirname,
	entry: {
		main: "./src/index.js"
	},
	resolve: {
		alias: {
			"core-js": path.dirname(require.resolve("core-js"))
		}
	},
	builtins: {
		presetEnv: {
			targets: ["> 0.01%", "not dead", "not op_mini all"],
			mode: "usage",
			coreJs: "3.26"
		},
		html: [
			{
				template: "./index.html"
			}
		]
	}
};
module.exports = config;
