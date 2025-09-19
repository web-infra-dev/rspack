/** @type {import("@rspack/core").Configuration} */
module.exports = {
	performance: {
		hints: false
	},
	optimization: {
		splitChunks: {
			minSize: 1
		},
		chunkIds: "named"
	}
};
