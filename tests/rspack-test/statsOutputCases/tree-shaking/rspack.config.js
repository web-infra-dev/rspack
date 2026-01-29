/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	entry: "./index",
	output: {
		filename: "bundle.js"
	},
	optimization: {
		concatenateModules: false
	},
	stats: {
		assets: true,
		chunkModules: false,
		modules: true,
		providedExports: true,
		usedExports: true
	}
};
