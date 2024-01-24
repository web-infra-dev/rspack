/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: 'production',
	entry: {
		"main": "./index.js"
	},
	experiments: {
		rspackFuture: {
			newTreeshaking: true,
		}
	},
	optimization: {
		concatenateModules: true,
		usedExports: true,
		sideEffects: true,
		providedExports: true,
		minimize: false
	}
};
