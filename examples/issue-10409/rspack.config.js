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
		mangleExports: false,
		moduleIds: 'named',
		concatenateModules: true,
		usedExports: true,
		sideEffects: true,
		providedExports: true,
		minimize: false,
		chunkIds: 'named'

	}
};
