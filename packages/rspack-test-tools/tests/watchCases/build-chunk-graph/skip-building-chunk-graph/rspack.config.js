/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization: {
		sideEffects: true,
		usedExports: true,
		innerGraph: true
	},
	experiments: {
		incremental: {
			buildChunkGraph: false
		}
	},
	module: {
		rules: [
			{
				test: /re-exports\.js$/,
				sideEffects: false
			}
		]
	}
};
