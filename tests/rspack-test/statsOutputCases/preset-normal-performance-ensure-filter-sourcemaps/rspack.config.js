/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	devtool: "source-map",
	performance: {
		hints: "warning"
	},
	entry: "./index",
	stats: {
		assets: true,
		modules: true,
		hash: false,
		colors: true
	}
};
