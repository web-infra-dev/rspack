/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	entry: "./index",
	stats: {
		assets: true,
		modules: true,
		colors: false
	}
};
