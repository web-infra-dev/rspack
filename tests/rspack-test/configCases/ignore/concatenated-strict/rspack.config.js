/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.mjs",
	resolve: {
		alias: {
			"./ignored-module": false
		}
	},
	output: {
		iife: false
	},
	optimization: {
		concatenateModules: true
	}
};
