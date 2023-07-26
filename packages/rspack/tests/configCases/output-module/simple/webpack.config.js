/** @type {import("@rspack/core").Configuration} */
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
	}
	// target: "node14"
};
