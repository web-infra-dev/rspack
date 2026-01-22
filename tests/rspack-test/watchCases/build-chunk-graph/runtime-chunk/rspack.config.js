/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		filename: "[name].js"
	},
	optimization: {
		runtimeChunk: true
	},
	incremental: {
		buildChunkGraph: true
	}
};
