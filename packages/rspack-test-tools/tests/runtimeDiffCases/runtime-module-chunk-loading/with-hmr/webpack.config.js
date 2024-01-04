const { HotModuleReplacementPlugin } = require("webpack");

module.exports = {
	entry: {
		main: "./src/a.js"
	},
	output: {
		filename: "[name].js",
		chunkLoading: "import",
		chunkFormat: "module",
		enabledChunkLoadingTypes: ["import"]
	},
	optimization: {
		runtimeChunk: {
			name: "bundle"
		}
	},
	target: "node",
	plugins: [new HotModuleReplacementPlugin()]
};
