const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		main: ["./index.js"]
	},
	output: {
		enabledChunkLoadingTypes: ["import", "async-node"]
	},
	plugins: [
		/** @param {import('@rspack/core').Compiler} compiler  */
		compiler => {
			rspack.javascript.EnableChunkLoadingPlugin.setEnabled(compiler, "custom");

			compiler.hooks.initialize.tap("test", () => {
				rspack.javascript.EnableChunkLoadingPlugin.checkEnabled(
					compiler,
					"custom"
				);
				rspack.javascript.EnableChunkLoadingPlugin.checkEnabled(
					compiler,
					"import"
				);
				rspack.javascript.EnableChunkLoadingPlugin.checkEnabled(
					compiler,
					"async-node"
				);
				expect(() =>
					rspack.javascript.EnableChunkLoadingPlugin.checkEnabled(
						compiler,
						"non-existing"
					)
				).toThrowErrorMatchingSnapshot();
			});
		}
	]
};
