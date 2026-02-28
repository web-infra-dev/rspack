const { rspack } = require("@rspack/core");

/** @type {function(any, any): import("@rspack/core").Configuration[]} */
module.exports = (env, { testPath }) => {
	return {
		externals: [/.*foo.*/],
		externalsType: "module",
		output: {
			module: true,
			chunkFormat: "module",
			filename: "[name].mjs"
		},
		optimization: {
			minimize: true,
			concatenateModules: true
		},
		plugins: [
			new rspack.CopyRspackPlugin({
				patterns: ["./a/**/*", "./_a/**/*"]
			})
		]
	};
};
