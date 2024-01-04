const { HotModuleReplacementPlugin } = require("webpack");

module.exports = {
	entry: {
		main: "./src/a.js"
	},
	output: {
		filename: "[name].js",
		chunkLoading: "require",
		enabledChunkLoadingTypes: ["require"]
	},
	optimization: {
		runtimeChunk: {
			name: "bundle"
		}
	},
	target: "node",
	plugins: [new HotModuleReplacementPlugin()]
};
