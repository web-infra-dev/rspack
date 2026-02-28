/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	output: {
		module: true,
		library: { type: "module" },
		iife: false,
		chunkFormat: "module",
		filename: "bundle0.mjs"
	},
	node: false,
	target: "node"
};
