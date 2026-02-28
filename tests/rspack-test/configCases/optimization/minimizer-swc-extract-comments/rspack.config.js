const { SwcJsMinimizerRspackPlugin } = require("@rspack/core");

/**
 * @type {import("@rspack/core").Configuration}
 */
module.exports = {
	optimization: {
		minimize: true,
		minimizer: [
			new SwcJsMinimizerRspackPlugin({
				extractComments: {},
				minimizerOptions: {
					format: {
						comments: "all"
					}
				}
			})
		]
	}
};
