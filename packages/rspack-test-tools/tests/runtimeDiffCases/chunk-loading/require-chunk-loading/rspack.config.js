/** @type {import("@rspack/core").Configuration} */
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
	target: "node"
};
