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
				).toThrowErrorMatchingInlineSnapshot(
					`Chunk loading type "non-existing" is not enabled. EnableChunkLoadingPlugin need to be used to enable this type of chunk loading. This usually happens through the "output.enabledChunkLoadingTypes" option. If you are using a function as entry which sets "chunkLoading", you need to add all potential chunk loading types to "output.enabledChunkLoadingTypes". These types are enabled: custom, import, async-node`
				);
			});
		}
	]
};
