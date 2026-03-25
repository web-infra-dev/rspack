/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	cache: true,
	output: {
		pathinfo: true
	},
	stats: {
		optimizationBailout: true,
		orphanModules: true
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
