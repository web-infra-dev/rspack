/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		"index": "./index.js",
	},
	output: {
		filename: `[name].js`,
		module: true,
		libraryTarget: "modern-module",
		iife: false,
		chunkFormat: "module",
	},
	externalsType: 'module-import',
	experiments: {
		outputModule: true
	},
	externals: "external-module",
	optimization: {
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
					const bundle = Object.values(assets)[0]._value
					expect(bundle).toContain(`var __webpack_exports__cjsInterop = (foo_default());
var __webpack_exports__defaultImport = external_external_module_namespaceObject["default"];
var __webpack_exports__namedImport = external_external_module_namespaceObject.namedImport;`)
				});
			};
			this.hooks.compilation.tap("testcase", handler);
		}
	]
};
