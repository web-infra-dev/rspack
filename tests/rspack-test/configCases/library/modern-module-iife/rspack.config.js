/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		index: "./index.js"
	},
	output: {
		filename: `[name].js`,
		iife: true,
		asyncChunks: false,
		library: {
			type: "modern-module"
		}
	},
	optimization: {
		minimize: true,
		moduleIds: "named"
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
					expect(bundle).toContain(
						`(()=>{"use strict";console.log("foo: ","foo")})();`
					);
				});
			};
			this.hooks.compilation.tap("testcase", handler);
		}
	]
};
