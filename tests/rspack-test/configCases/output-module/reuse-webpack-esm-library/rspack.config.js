const path = require("path");
/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	devtool: "eval",
	optimization: {
		concatenateModules: false
	},
	resolve: {
		alias: {
			react: path.resolve(__dirname, "react")
		}
	}
};
