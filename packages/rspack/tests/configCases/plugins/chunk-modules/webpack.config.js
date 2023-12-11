module.exports = {
	plugins: [
		function plugin(compiler) {
			compiler.hooks.compilation.tap("plugin", compilation => {
				compilation.hooks.processAssets.tap("plugin", () => {
					const chunkModules = {};
					for (let chunk of compilation.chunks) {
						const modules = [
							...compilation.chunkGraph.getChunkModulesIterable(chunk)
						];
						const entryModules = [
							...compilation.chunkGraph.getChunkEntryModulesIterable(chunk)
						];
						chunkModules[chunk.id] = { modules, entryModules };
					}
					compilation.emitAsset(
						"data.json",
						new compiler.webpack.sources.RawSource(
							JSON.stringify(chunkModules, null, 2)
						)
					);
				});
			});
		}
	]
};
