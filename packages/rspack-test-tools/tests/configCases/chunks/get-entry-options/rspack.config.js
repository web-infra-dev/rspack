/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "node",
	entry: {
		main: {
			import: "./index.js",
			chunkLoading: "async-node"
		}
	},
	plugins: [
		{
			/**@param {import("@rspack/core").Compiler} compiler */
			apply(compiler) {
				compiler.hooks.thisCompilation.tap("test", compilation => {
					compilation.hooks.afterSeal.tap("test", () => {
						let entrypoint = compilation.entrypoints.get("main");

						entrypoint.chunks.forEach(chunk => {
							const entryOptions = chunk.getEntryOptions();

							expect(entryOptions).not.toBeUndefined();
							expect(entryOptions.chunkLoading).toBe("async-node");
						});
					});
				});
			}
		}
	]
};
