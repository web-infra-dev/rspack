/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	/// DIFF: rspack uses cache: true to enable memory cache
	// cache: {
	// 	type: "memory"
	// },
	cache: true,
	output: {
		filename: "bundle.js?[contenthash]"
	}
};
