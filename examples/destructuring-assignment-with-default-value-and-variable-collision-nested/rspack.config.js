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
		minimize: false
	}
};
