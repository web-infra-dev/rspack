/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	output: {
		libraryTarget: "module",
		iife: false,
		chunkFormat: "module",
		filename: "bundle0.mjs"
	},
	experiments: {
		outputModule: true,
		rspackFuture: {
			newTreeshaking: false
		}
	},
	target: "node"
};
