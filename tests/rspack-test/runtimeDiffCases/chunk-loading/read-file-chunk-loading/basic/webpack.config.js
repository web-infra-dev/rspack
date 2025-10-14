/** @type {import("webpack").Configuration} */
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
