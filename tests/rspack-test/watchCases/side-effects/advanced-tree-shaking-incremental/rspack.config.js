/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	cache: true,
	output: {
		pathinfo: true
	},
	optimization: {
		minimize: false,
		sideEffects: true,
		providedExports: true,
		concatenateModules: false
	},
	experiments: {
		advancedTreeShaking: true,
		cache: {
			type: "memory"
		}
	}
};
