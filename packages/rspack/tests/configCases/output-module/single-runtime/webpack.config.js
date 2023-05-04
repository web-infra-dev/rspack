/** @type {import("../../../../dist").Configuration} */
module.exports = {
	output: {
		filename: "[name].js",
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
