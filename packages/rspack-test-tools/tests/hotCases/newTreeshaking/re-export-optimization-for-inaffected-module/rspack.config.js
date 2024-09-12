/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization: {
		sideEffects: true,
		providedExports: true,
	},
	experiments: {
		rspackFuture: {
			newIncremental: true
		}
	}
};
