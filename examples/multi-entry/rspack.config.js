const rspack = require("@rspack/core");
/** @type {import('@rspack/cli').Configuration} */
const config = {
	mode: "development",
	entry: {
		index: "./index.js",
		second: "./second.js"
	},
	output: {
		publicPath: "http://localhost:3000"
	},
	plugins: [new rspack.HtmlRspackPlugin()]
};
module.exports = config;
