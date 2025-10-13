const { SwcJsMinimizerRspackPlugin } = require("@rspack/core");

/** @type {import("@rspack/coresrc/index").RspackOptions} */
module.exports = {
	plugins: [
		new SwcJsMinimizerRspackPlugin({
			minimizerOptions: {
				format: {
					asciiOnly: true
				}
			}
		})
	]
};
