/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "async-node",
	externals: ["path", "fs"],
	externalsType: "module",
	output: {
		chunkFormat: "module",
		filename: "[name].mjs",
		library: {
			type: "modern-module"
		}
	},
	experiments: {
		outputModule: true
	}
};
