/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		libraryTarget: "module",
		iife: false,
		chunkFormat: "module",
		filename: "bundle0.mjs"
	},
	experiments: {
		outputModule: true
	},
	target: "node",
	optimization: {
		minimize: true
	}
};
