/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	entry: {
		index: "./index.js",
		entry: "./entry.js"
	},
	target: "web",
	output: {
		filename: "[name].js"
	},
	externals: ["external"],
	stats: {
		assets: false,
		modules: true,
		orphanModules: true,
		optimizationBailout: true
	}
};
