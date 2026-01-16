/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		index: "./index.js"
	},
	output: {
		filename: `[name].js`,
		module: true,
		library: { type: "modern-module" },
		iife: false,
		chunkFormat: "module"
	},
	externalsType: "module-import",
	experiments: {
		outputModule: true
	},
	externals: "external-module",
	optimization: {
		avoidEntryIife: true,
		concatenateModules: true,
		minimize: false
	},
	plugins: [
		function () {
			/**
			 * @param {import("@rspack/core").Compilation} compilation compilation
			 * @returns {void}
			 */
			const handler = compilation => {
				compilation.hooks.afterProcessAssets.tap("testcase", assets => {
					const bundle = Object.values(assets)[0]._value;
					expect(bundle)
						.toContain(`var __webpack_exports__cjsInterop = (foo_default());
export { external_module as defaultImport, namedImport, __webpack_exports__cjsInterop as cjsInterop };`);
					expect(bundle).toContain(
						'import external_module, { namedImport } from "external-module";'
					);
				});
			};
			this.hooks.compilation.tap("testcase", handler);
		}
	]
};
