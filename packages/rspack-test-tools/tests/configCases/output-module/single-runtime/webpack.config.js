/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		filename: "[name].mjs",
		chunkFormat: "module",
		chunkLoading: "import",
		library: {
			type: "module"
		}
	},
	experiments: {
		outputModule: true
	},
	optimization: {
		runtimeChunk: true
	}
	// target: "node14"
};
