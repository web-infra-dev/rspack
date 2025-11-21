const {
	experiments: { VirtualModulesPlugin }
} = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		/**
		 * @param {import("@rspack/core").Compiler} compiler
		 */
		function testWatch(compiler) {
			const virtualModules = new VirtualModulesPlugin({
				"dynamic-module.js": 'export const step = "0";'
			});

			virtualModules.apply(compiler);

			let initialized = false;
			compiler.hooks.afterDone.tap("test-watch", function (_stats) {
				if (!initialized) {
					initialized = true;
					setTimeout(() => {
						virtualModules.writeModule(
							"dynamic-module.js",
							'export const step = "1";'
						);
					}, 500);
				}
			});
		}
	]
};
