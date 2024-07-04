/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "none",
	entry: "./main.js",
	optimization: {
		usedExports: true,
		sideEffects: true,
		concatenateModules: true
	},
	stats: {
		orphanModules: true,
		nestedModules: true,
		usedExports: true,
		reasons: true
	}
};
