/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "web",
	output: {
		chunkFilename: "[name].web.js",
		crossOriginLoading: "anonymous"
	},
	performance: {
		hints: false
	},
	optimization: {
		minimize: false
	}
};
