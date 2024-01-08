const { HotModuleReplacementPlugin } = require("webpack");

module.exports = {
	entry: {
		main: "./src/a.js"
	},
	output: {
		filename: "[name].js",
		chunkLoading: "async-node",
		enabledChunkLoadingTypes: ["async-node"]
	},
	optimization: {
		runtimeChunk: {
			name: "bundle"
		}
	},
	target: "node",
	plugins: [new HotModuleReplacementPlugin()]
};
