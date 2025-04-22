/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		index: "./index.js"
	},
	output: {
		filename: `[name].js`,
		module: true,
		libraryTarget: "modern-module",
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
var __webpack_exports__defaultImport = __WEBPACK_EXTERNAL_MODULE_external_module_43054e33__["default"];
var __webpack_exports__namedImport = __WEBPACK_EXTERNAL_MODULE_external_module_43054e33__.namedImport;`);
				});
			};
			this.hooks.compilation.tap("testcase", handler);
		}
	]
};
