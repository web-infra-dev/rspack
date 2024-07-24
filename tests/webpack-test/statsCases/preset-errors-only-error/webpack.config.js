const LogTestPlugin = require("../../helpers/LogTestPlugin");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	entry: "./index",
	stats: "errors-only",
	infrastructureLogging: {
		level: "error"
	},
	plugins: [new LogTestPlugin()]
};
