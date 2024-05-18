const path = require("path");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		main: "./index.js"
	},
	resolve: {
		tsConfig: {
			configFile: path.resolve(__dirname, "./tsconfig.json"),
			references: "auto"
		}
	}
};
