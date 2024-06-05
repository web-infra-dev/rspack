const { HotModuleReplacementPlugin } = require("webpack");

/** @type {import("webpack").Configuration} */
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
