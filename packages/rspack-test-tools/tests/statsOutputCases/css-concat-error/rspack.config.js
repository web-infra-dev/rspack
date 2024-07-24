const path = require("path");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization: {
		concatenateModules: true,
		minimize: false
	},
	experiments: {
		css: true,
	}
};
