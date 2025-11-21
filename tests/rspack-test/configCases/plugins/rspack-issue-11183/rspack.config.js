const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		filename: "[name].bundle"
	},
	optimization: {
		minimize: true,
		minimizer: [
			new rspack.SwcJsMinimizerRspackPlugin({
				test: /\.bundle$/
			})
		]
	}
};
