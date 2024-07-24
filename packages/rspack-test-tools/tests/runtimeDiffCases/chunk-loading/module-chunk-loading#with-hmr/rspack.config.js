const { HotModuleReplacementPlugin } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
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
