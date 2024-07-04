const { HotModuleReplacementPlugin } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	devtool: false,
	experiments: { topLevelAwait: true },
	optimization: { usedExports: false, sideEffects: false },
	plugins: [new HotModuleReplacementPlugin()]
};
