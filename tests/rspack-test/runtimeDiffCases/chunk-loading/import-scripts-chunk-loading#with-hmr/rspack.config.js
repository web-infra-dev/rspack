const { HotModuleReplacementPlugin } = require("webpack");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		chunkLoading: "import-scripts",
		enabledChunkLoadingTypes: ["import-scripts"]
	},
	plugins: [new HotModuleReplacementPlugin()]
};
