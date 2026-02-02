/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		module: true,
		filename: "[name].mjs",
		chunkFormat: "module",
		chunkLoading: "import",
		library: {
			type: "module"
		}
	},
	experiments: {
		},
	optimization: {
		runtimeChunk: true
	}
	// target: "node14"
};
