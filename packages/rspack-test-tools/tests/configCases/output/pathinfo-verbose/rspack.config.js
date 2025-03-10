/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "node",
	entry: {
		index: "./index.js",
		sut: "./sut.js"
	},
	output: {
		pathinfo: "verbose",
		filename: "[name].js"
	},
	optimization: {
		minimize: false,
		chunkIds: "named",
		concatenateModules: false
	}
};
