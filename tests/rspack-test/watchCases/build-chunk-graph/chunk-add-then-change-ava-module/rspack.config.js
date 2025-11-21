/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization: {
		splitChunks: false,
		sideEffects: false
	},
	experiments: {
		incremental: {
			buildChunkGraph: true
		}
	}
};
