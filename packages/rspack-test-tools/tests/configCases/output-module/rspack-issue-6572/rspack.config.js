/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	optimization: {
		minimize: false,
	},
	output: {
    library: {
      type: "module",
    },
		filename: "[name].mjs",
    module: true,
    chunkFormat: "module",
    chunkLoading: "import",
	}
};
