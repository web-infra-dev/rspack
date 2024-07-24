const LogTestPlugin = require("../../helpers/LogTestPlugin");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	entry: "./index",
	stats: "summary",
	plugins: [new LogTestPlugin()]
};
