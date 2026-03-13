const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization: {
		minimize: true,
		minimizer: [
			new rspack.SwcJsMinimizerRspackPlugin({
				extractComments: {
					// Test case sensitivity - this should NOT match 'license' (lowercase)
					condition: /LICENSE/,
				},
			}),
		],
	},
};
