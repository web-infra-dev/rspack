const MyStillValidModulePlugin = require("./plugins/MyStillValidModulePlugin");

/** @type {import("@rspack/core").Configuration} */
const config = {
	context: __dirname,
	mode: "development",
	entry: {
		main: "./src/index.js"
	},
	plugins: [new MyStillValidModulePlugin()]
};
module.exports = config;
