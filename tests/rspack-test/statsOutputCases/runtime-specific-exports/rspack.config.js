/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./example.js",
	optimization: {
		usedExports: true,
		providedExports: true
	},
	stats: {
		assets: true,
		modules: true,
		usedExports: true,
		providedExports: true
	}
};
