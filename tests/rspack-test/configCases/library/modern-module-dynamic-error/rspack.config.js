/** @type {import("@rspack/core").Configuration} */

module.exports = {
	entry: {
		index: "./index.js"
	},
	output: {
		module: true,
		filename: `[name].js`,
		chunkFilename: `async.js`,
		library: {
			type: "modern-module"
		},
		iife: false,
		chunkFormat: "module",
		chunkLoading: "import"
	},
	externalsType: "module-import",
	optimization: {
		concatenateModules: true,
		avoidEntryIife: true,
		minimize: false
	}
};
