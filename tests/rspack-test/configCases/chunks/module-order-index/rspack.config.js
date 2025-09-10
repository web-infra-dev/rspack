/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "node",
	entry: {
		main: "./index.js"
	},
	plugins: [
		{
			/**@param {import("@rspack/core").Compiler} compiler */
			apply(compiler) {
				compiler.hooks.thisCompilation.tap("test", compilation => {
					compilation.hooks.afterSeal.tap("test", () => {
						let entrypoint = compilation.entrypoints.get("main");

						compilation.chunkGraph
							.getChunkModules(entrypoint.chunks[0])
							.forEach(m => {
								expect(entrypoint.getModulePreOrderIndex(m)).toBeDefined();
								expect(entrypoint.getModulePostOrderIndex(m)).toBeDefined();
							});
					});
				});
			}
		}
	]
};
