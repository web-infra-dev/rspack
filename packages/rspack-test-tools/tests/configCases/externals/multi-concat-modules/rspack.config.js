/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "node",
	output: {
		module: true,
		chunkFormat: "module",
		filename: "[name].mjs"
	},
	optimization: {
		concatenateModules: true
	},
	externals: {
		"externals-1/foo": "fs",
		"externals-2/foo": "fs?1"
	},
	externalsType: "module",
	experiments: {
		outputModule: true
	}
};
