/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	output: {
		filename: "bundle0.mjs",
		module: true,
	},
	experiments: {
		module: true,
	},
	devtool: "source-map"
};
