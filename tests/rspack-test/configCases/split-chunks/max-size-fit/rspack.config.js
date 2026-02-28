/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "web",
	entry: "./index.js",
	output: {
		filename: "[id].js"
	},
	ignoreWarnings: [/\.*/],
	optimization: {
		chunkIds: "named",
		moduleIds: "named",
		splitChunks: {
			chunks: "all",
			minSize: 100 * 1024,
			maxSize: 200 * 1024
		}
	}
};
