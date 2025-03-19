/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "node",
	entry: {
		index: "./index.js",
		sut: "./sut.js"
	},
	output: {
		pathinfo: true,
		filename: "[name].js"
	},
	optimization: {
		chunkIds: "named",
		concatenateModules: false
	}
};
