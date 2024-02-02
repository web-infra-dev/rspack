var webpack = require("../../../../");
/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		strictThisContextOnImports: true
	},
	optimization: {
		concatenateModules: true
	}
};
