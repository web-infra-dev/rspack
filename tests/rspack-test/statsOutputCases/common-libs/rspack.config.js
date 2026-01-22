/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	entry: {
		react: "./react"
	},
	optimization: {
		minimize: true,
		chunkIds: "named"
	},
	stats: {
		assets: true,
		modules: true,
	}
};
