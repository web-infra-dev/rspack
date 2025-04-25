const { supportsImportFn } = require("@rspack/test-tools");
/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	module: {
		rules: [
			{
				test: /\.s[ac]ss$/i,
				use: [{ loader: "sass-loader", parallel: true, options: {} }],
				type: "css"
			}
		]
	},
	experiments: {
		parallelLoader: supportsImportFn()
	}
};
