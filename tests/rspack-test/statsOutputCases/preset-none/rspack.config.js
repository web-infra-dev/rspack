const LogTestPlugin = require("@rspack/test-tools/helper/legacy/LogTestPlugin");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	entry: "./index",
	stats: false,
	plugins: [new LogTestPlugin()]
};
