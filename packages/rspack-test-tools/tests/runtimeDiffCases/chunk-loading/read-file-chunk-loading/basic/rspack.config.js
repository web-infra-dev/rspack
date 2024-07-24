/** @type {import("@rspack/core").Configuration} */
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
	target: "node"
};
