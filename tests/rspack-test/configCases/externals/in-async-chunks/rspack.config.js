/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "async-node",
	externals: ["path", "fs"],
	externalsType: "module",
	output: {
		module: true,
		chunkFormat: "module",
		filename: "[name].mjs",
		library: {
			type: "modern-module"
		}
	},
};
