/** @type {import("../../../../").Configuration} */
module.exports = {
	output: {
		libraryTarget: "module",
		iife: false,
		chunkFormat: "module",
		filename: "main.js"
	},
	experiments: {
		outputModule: true
	},
	target: "node",
	optimization: {
		minimize: true
	}
};
