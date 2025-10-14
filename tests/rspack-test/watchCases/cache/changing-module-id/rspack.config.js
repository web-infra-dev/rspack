/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	/// DIFF: rspack uses cache: true to enable memory cache
	// cache: {
	// 	type: "memory"
	// },
	cache: true,
	optimization: {
		sideEffects: false,
		providedExports: false
	},
	module: {
		rules: [
			{
				test: /other-layer/,
				layer: "other-layer"
			}
		]
	},
	experiments: {
		// cacheUnaffected: true,
	}
};
