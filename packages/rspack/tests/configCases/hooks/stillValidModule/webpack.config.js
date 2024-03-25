const MyStillValidModulePlugin = require("./plugins/MyStillValidModulePlugin");

/** @type {import('@rspack/cli').Configuration} */
const config = {
	context: __dirname,
	mode: "development",
	entry: {
		main: "./src/index.js"
	},
	plugins: [new MyStillValidModulePlugin()]
};
module.exports = config;
