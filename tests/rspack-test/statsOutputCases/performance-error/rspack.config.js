/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	entry: "./index",
	stats: {
		assets: true,
		modules: true,
		colors: true,
		hash: false,
		entrypoints: true
	},
	performance: {
		hints: "error"
	}
};
