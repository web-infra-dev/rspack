/** @type {import("../../../../").Configuration} */
module.exports = {
	output: {
		libraryTarget: "module",
		iife: false,
		chunkFormat: "module",
		filename: "main.js"
	},
	optimization: {
		providedExports: true
	},
	experiments: {
		outputModule: true,
		rspackFuture: {
			newTreeshaking: false
		}
	},
	target: "node"
};
