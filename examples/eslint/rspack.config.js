const EslintPlugin = require("eslint-rspack-plugin");
/**
 * @type {import('@rspack/cli').Configuration}
 */
module.exports = {
	mode: "development",
	context: __dirname,
	stats: { all: true },
	entry: {
		main: "./src/index.js"
	},
	plugins: [new EslintPlugin()]
};
