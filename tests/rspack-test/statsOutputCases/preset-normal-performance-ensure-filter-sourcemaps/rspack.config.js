/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	devtool: "source-map",
	performance: {
		hints: "warning"
	},
	entry: "./index",
	stats: {
		hash: false,
		colors: true
	}
};
