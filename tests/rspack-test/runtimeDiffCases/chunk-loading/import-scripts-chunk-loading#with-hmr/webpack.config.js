const { HotModuleReplacementPlugin } = require("webpack");

/** @type {import("webpack").Configuration} */
module.exports = {
	output: {
		chunkLoading: "import-scripts",
		enabledChunkLoadingTypes: ["import-scripts"]
	},
	plugins: [new HotModuleReplacementPlugin()]
};
