/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization: {
		usedExports: true
	},
	experiments: {
		rspackFuture: {
			newTreeshaking: true
		}
	}
};
