const { supportsImportFn } = require("@rspack/test-tools");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		rules: [
			{
				test: /c\.js$/,
				use: [{ loader: "loader2", parallel: true, options: {} }]
			},
			{
				test: /d\.js$/,
				use: [{ loader: "loader3", parallel: true, options: {} }]
			}
		]
	},
	experiments: {
		parallelLoader: supportsImportFn()
	}
};
