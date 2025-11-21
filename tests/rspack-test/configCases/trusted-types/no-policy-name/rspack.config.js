/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "web",
	output: {
		// TODO should be `[name].web.js`
		chunkFilename: "no-trusted-types-policy-name.web.js",
		crossOriginLoading: "anonymous"
	},
	// performance: {
	// 	hints: false
	// },
	optimization: {
		minimize: false
	}
};
