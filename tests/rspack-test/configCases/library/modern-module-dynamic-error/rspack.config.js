/** @type {import("@rspack/core").Configuration} */

module.exports = {
	entry: {
		index: "./index.js"
	},
	output: {
		filename: `[name].js`,
		chunkFilename: `async.js`,
		module: true,
		library: {
			type: "modern-module"
		},
		iife: false,
		chunkFormat: "module",
		chunkLoading: "import"
	},
	externalsType: "module-import",
	experiments: {
		outputModule: true
	},
	optimization: {
		concatenateModules: true,
		avoidEntryIife: true,
		minimize: false
	}
};
