/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	resolve: {
		alias: {
			"./ignored-module": false
		}
	},
	devtool: "source-map"
};
