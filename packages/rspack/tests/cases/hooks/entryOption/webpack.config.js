/** @type {import('@rspack/cli').Configuration} */
const MyEntryOptionPlugin = require("./plugins/MyEntryOptionPlugin");
const config = {
	context: __dirname,
	mode: "development",
	entry: {
		main: "./src/index.js",
		test: "./src/index2.js"
	},
	plugins: [new MyEntryOptionPlugin()]
};
module.exports = config;
