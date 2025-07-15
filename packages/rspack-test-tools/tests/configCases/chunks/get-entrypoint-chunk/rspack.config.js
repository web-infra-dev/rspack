/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	output: {
		filename: "[name].js"
	},
	optimization: {
		runtimeChunk: true
	},
	plugins: [
		{
			/**@param {import("@rspack/core").Compiler} compiler */
			apply(compiler) {
				compiler.hooks.thisCompilation.tap("test", compilation => {
					compilation.hooks.afterSeal.tap("test", () => {
						const entrypoint = compilation.entrypoints.get("main");
						const entrypointChunk = entrypoint.getEntrypointChunk();
						expect(entrypointChunk.name).toBe("main");
						const chunks = entrypointChunk.getAllReferencedChunks();
						expect([...chunks].map(c => c.name)).toEqual([
							"runtime~main",
							"main",
							"async"
						]);
					});
				});
			}
		}
	]
};
