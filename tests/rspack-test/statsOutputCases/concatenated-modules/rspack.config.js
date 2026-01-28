/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index",
	optimization: {
		concatenateModules: true,
		minimize: false
	},
	stats: {
		assets: true,
		modules: true,
	}
};
