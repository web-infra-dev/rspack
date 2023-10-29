const rspack = require("@rspack/core");
/** @type {import('@rspack/cli').Configuration} */
const config = {
	mode: "development",
	entry: "./index.js",
	plugins: [new rspack.HtmlRspackPlugin()]
};
module.exports = config;
