/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		filename: "[name].js"
	},
	mode: "development",
	entry: "./index.js",
	optimization: {
		chunkIds: "named",
		splitChunks: {
			// hidePathInfo: false,
			minSize: 50,
			maxSize: 100
		}
	}
};
