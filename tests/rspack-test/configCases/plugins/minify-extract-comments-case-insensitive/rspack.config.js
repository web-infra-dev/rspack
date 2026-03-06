const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization: {
		minimize: true,
		minimizer: [
			new rspack.SwcJsMinimizerRspackPlugin({
				extractComments: {
					// Test case insensitivity - should match both 'LICENSE' and 'license'
					condition: /LICENSE/i,
				},
			}),
		],
	},
};
