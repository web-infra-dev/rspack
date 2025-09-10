/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		main: "./src/a.js"
	},
	output: {
		filename: "[name].js",
		chunkLoading: "jsonp",
		enabledChunkLoadingTypes: ["jsonp"]
	},
	optimization: {
		runtimeChunk: {
			name: "bundle"
		}
	},
	target: "web"
};
