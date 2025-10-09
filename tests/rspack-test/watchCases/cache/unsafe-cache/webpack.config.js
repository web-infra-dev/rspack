/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	/// DIFF: rspack uses cache: true to enable memory cache
	// cache: {
	// 	type: "memory"
	// },
	cache: true,
	module: {
		// unsafeCache: true
	},
	externals: {
		external: "var 123"
	}
};
