/** @type {import("@rspack/core").Configuration} */
module.exports = {
	experiments: {
		rspackFuture: {
			newTreeshaking: true,
		}
	},
	optimization: {
		concatenateModules: true
	}
};
