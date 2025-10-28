/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	/// DIFF: rspack uses cache: true to enable memory cache
	// cache: {
	// 	type: "memory"
	// },
	cache: true,
	module: {
		rules: [
			{
				test: /\.png$/,
				type: "asset/inline"
			},
			{
				test: /\.jpg$/,
				type: "asset/resource"
			},
			{
				test: /\.svg$/,
				type: "asset"
			}
		]
	},
	optimization: {
		concatenateModules: true
	}
};
