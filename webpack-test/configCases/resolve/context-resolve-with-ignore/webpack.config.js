const path = require("path");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	resolve: {
		modules: [path.resolve(__dirname, "a"), path.resolve(__dirname, "b")],
		alias: {
			[path.resolve(__dirname, "a", "foo")]: false
		}
	}
};
