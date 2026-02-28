/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	entry: "./index",
	performance: {
		hints: "warning"
	},
	stats: {
		assets: true,
		modules: true,
		hash: false,
		colors: true
	}
};
