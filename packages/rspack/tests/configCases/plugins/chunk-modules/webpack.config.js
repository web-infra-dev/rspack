const path = require("path");

module.exports = {
	plugins: [
		function plugin(compiler) {
			function moduleStringify(module) {
				return {
					resource: module.resource
						? normalizePathLike(module.resource)
						: undefined,
					context: module.context
						? normalizePathLike(module.context)
						: undefined,
					identifier: normalizePathLike(module.identifier())
				};
			}

			function normalizePathLike(value) {
				return value
					.replace(compiler.context, "<COMPILER_CONTEXT>")
					.replace(/\\/g, "/");
			}

			compiler.hooks.compilation.tap("plugin", compilation => {
				compilation.hooks.processAssets.tap("plugin", () => {
					const chunkModules = {};
					for (let chunk of compilation.chunks) {
						const modules = [
							...compilation.chunkGraph.getChunkModulesIterable(chunk)
						].map(moduleStringify);
						const entryModules = [
							...compilation.chunkGraph.getChunkEntryModulesIterable(chunk)
						].map(moduleStringify);
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
