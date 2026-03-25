/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	cache: true,
	output: {
		pathinfo: true
	},
	stats: {
		orphanModules: true
	},
	optimization: {
		minimize: false,
		sideEffects: true,
		innerGraph: true,
		usedExports: true,
		concatenateModules: false
	},
	experiments: {
		advancedTreeShaking: true,
		cache: {
			type: "memory"
		}
	}
};
