const EslintPlugin = require("eslint-rspack-plugin");
/** @type {import('@rspack/cli').Configuration} */
const config = {
	mode: "development",
	context: __dirname,
	entry: {
		main: "./src/index.js"
	},
	plugins: [new EslintPlugin()]
};
module.exports = config;
