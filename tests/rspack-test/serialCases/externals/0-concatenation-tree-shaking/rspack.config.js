const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	externals: [/\.\/externals\/.*/],
	externalsType: "module",
	output: {
		module: true,
		chunkFormat: "module",
		filename: "[name].mjs",
		library: {
			type: "modern-module"
		}
	},
	optimization: {
		avoidEntryIife: true
	},
	experiments: {
		outputModule: true
	},
	plugins: [
		new rspack.CopyRspackPlugin({
			patterns: ["./externals/**/*"]
		})
	]
};
