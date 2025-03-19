const PLUGIN_NAME = "plugin";

class Plugin {
	/**
	 * @param {import("@rspack/core").Compiler} compiler
	 */
	apply(compiler) {
		compiler.hooks.make.tap(PLUGIN_NAME, compilation => {
			compilation.hooks.processAssets.tap(PLUGIN_NAME, () => {
				const entrypoint = Array.from(compilation.entrypoints.values())[0];
				const entrypointChunk = entrypoint.chunks[0];
				const entrypointModule =
					compilation.chunkGraph.getChunkModules(entrypointChunk)[0];
				expect(entrypointModule.modules[0].rawRequest).toBe("./foo");
				expect(entrypointModule.modules[1].rawRequest).toBe("./index.js");
			});
		});
	}
}

/**@type {import("@rspack/core").Configuration}*/
module.exports = {
	entry: "./index.js",
	plugins: [new Plugin()],
	optimization: {
		concatenateModules: true
	}
};
