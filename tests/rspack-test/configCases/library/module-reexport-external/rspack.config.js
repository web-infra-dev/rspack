"use strict";

/** @typedef {import("@rspack/core").Compilation} Compilation */

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "none",
	entry: { main: "./index.js", test: "./test" },
	output: {
		module: true,
		library: {
			type: "module"
		},
		filename: "[name].js",
		chunkFormat: "module"
	},
	experiments: {
		outputModule: true
	},
	resolve: {
		extensions: [".js"]
	},
	externalsType: "module",
	externals: ["external0"],
	optimization: {
		concatenateModules: true
	},
	plugins: [
		function apply() {
			/**
			 * @param {Compilation} compilation compilation
			 */
			const handler = (compilation) => {
				compilation.hooks.afterProcessAssets.tap("testcase", (assets) => {
					const source = assets["test.js"].source();
					// DIFF: 
					// expect(source).toContain("export const value");
					expect(source).toContain("export { __webpack_exports__value as value };");
				});
			};
			this.hooks.compilation.tap("testcase", handler);
		}
	]
};
