/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization: {
		moduleIds: "named"
	},
	output: {
		webassemblyModuleFilename: "[id].[hash].wasm"
	},
	experiments: {
		asyncWebAssembly: true
	}
};
