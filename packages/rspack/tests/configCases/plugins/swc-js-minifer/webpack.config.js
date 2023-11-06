const { SwcJsMinimizerRspackPlugin } = require("@rspack/core");

/** @type {import("../../../../src/index").RspackOptions} */
module.exports = {
	entry: {
		main: ["./index.js"],
		extract: ["./extract.js"]
	},
	plugins: [
		new SwcJsMinimizerRspackPlugin({
			extractComments: true,
			include: ["extract.js"]
		})
	]
};
