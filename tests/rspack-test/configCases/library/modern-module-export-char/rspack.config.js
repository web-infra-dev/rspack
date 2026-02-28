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
	externals: "external-module",
	optimization: {
		runtimeChunk: false
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
						.toContain(`var foo_default = /*#__PURE__*/__webpack_require__.n(foo);\nvar foo_default_0 = foo_default();`);
					expect(bundle).toContain('foo_default_0 as cjsInterop')
					expect(bundle).toContain(
						'export { default as defaultImport, namedImport } from "external-module";'
					);
				});
			};
			this.hooks.compilation.tap("testcase", handler);
		}
	]
};
