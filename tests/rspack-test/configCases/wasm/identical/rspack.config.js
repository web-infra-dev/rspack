const { CachedSource } = require("webpack-sources");
const { AsyncWebAssemblyModulesPlugin } = require("@rspack/core").wasm;

/** @typedef {import("@rspack/core").Compiler} Compiler */

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		rules: [
			{
				test: /\.wat$/,
				loader: "wast-loader",
				type: "webassembly/async"
			}
		]
	},
	output: {
		webassemblyModuleFilename: "[id].[hash].wasm"
	},
	experiments: {
		asyncWebAssembly: true
	},
	plugins: [
		/**
		 * @this {Compiler} compiler
		 */
		function () {
			this.hooks.compilation.tap("Test", compilation => {
				AsyncWebAssemblyModulesPlugin.getCompilationHooks(
					compilation
				).renderModuleContent.tap("Test", source => {
					// this is important to make each returned value a new instance
					return new CachedSource(source);
				});
			});
		}
	]
};
