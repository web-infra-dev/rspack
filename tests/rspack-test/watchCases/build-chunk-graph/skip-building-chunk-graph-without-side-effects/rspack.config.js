/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization: {
		sideEffects: true,
		usedExports: false,
		innerGraph: true
	},
	incremental: {
		buildChunkGraph: false
	},
	module: {
		rules: [
			{
				test: /\.js$/,
				sideEffects: false
			}
		]
	}
};
