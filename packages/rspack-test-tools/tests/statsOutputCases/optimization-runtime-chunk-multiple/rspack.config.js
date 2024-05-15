/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	entry: {
		e1: "./e1",
		e2: "./e2"
	},
	output: {
		filename: "[name].js"
	},
	stats: {
		all: false,
		reasons: true,
		chunks: true,
		entrypoints: true,
		chunkGroups: true,
		errors: true,
	},
	optimization: {
		runtimeChunk: "multiple"
	}
};
