const { HotModuleReplacementPlugin } = require("webpack");

module.exports = {
	output: {
		chunkLoading: "import-scripts",
		enabledChunkLoadingTypes: ["import-scripts"]
	},
	plugins: [new HotModuleReplacementPlugin()]
};
