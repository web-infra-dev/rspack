const path = require("path");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		main: "./index.js"
	},
	resolve: {
		tsConfigPath: path.resolve(__dirname, "./tsconfig.json")
	}
};
