const MySucceedModulePlugin = require("./plugins/MySucceedModulePlugin");

/** @type {import('@rspack/cli').Configuration} */
const config = {
	context: __dirname,
	mode: "development",
	entry: {
		main: "./src/index.js"
	},
	plugins: [new MySucceedModulePlugin()]
};

module.exports = config;
