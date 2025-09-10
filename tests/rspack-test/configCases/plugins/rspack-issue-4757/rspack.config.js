const { SwcJsMinimizerRspackPlugin } = require("@rspack/core");

/** @type {import("../../../../src/index").RspackOptions} */
module.exports = {
	plugins: [new SwcJsMinimizerRspackPlugin()]
};
