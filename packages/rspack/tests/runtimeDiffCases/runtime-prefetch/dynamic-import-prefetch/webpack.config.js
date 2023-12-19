/** @type {import("../../../../").Configuration} */
module.exports = {
	target: "web",
	output: {
		chunkFilename: "[name].js",
		crossOriginLoading: "anonymous"
	},
	module: {
		parser: {
			javascript: {
				dynamicImportPrefetch: true
			}
		}
	}
};
