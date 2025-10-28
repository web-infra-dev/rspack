/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	entry: "./index",
	stats: {
		all: false,
		assets: true,
		entrypoints: true,
		chunkGroupChildren: true,
		chunks: true
	}
};
