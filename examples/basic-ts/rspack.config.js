const path = require("path");

/** @type {import('@rspack/cli').Configuration} */
const config = {
	context: __dirname,
	mode: "development",
	entry: {
		main: "./src/index.ts"
	},
	resolve: {
		tsConfigPath: path.resolve(__dirname, "tsconfig.json")
	},
	builtins: {
		html: [
			{
				template: "./index.html"
			}
		]
	}
};
module.exports = config;
