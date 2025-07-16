const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	output: {
		filename: "[name].js"
	},

	optimization: {
		minimize: false,
		minimizer: [
			new rspack.SwcJsMinimizerRspackPlugin({
				minimizerOptions: {
					ecma: 2020,
					module: true
				}
			})
		]
	}
};
