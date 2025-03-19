/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	optimization: {
		concatenateModules: true,
		minimize: false
	},
	experiments: {
		css: true
	}
};
