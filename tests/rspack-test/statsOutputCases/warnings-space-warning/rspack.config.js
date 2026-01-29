/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	mode: "production",
	stats: {
		assets: true,
		modules: true,
		warningsSpace: 0,
		warnings: true
	}
};
