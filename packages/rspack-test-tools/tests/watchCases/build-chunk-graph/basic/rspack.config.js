/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization: {
		splitChunks: false
	},
	experiments: {
		incremental: {
			buildChunkGraph: true
		}
	}
};
