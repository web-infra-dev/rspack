/** @type {import("@rspack/core").Configuration} */
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

			function compareByIdentifier(a, b) {
				return a.identifier().localeCompare(b.identifier());
			}

			compiler.hooks.compilation.tap("plugin", compilation => {
				compilation.hooks.processAssets.tap("plugin", () => {
					const chunkModules = {};
					for (let chunk of compilation.chunks) {
						const modules = [
							...compilation.chunkGraph.getChunkModulesIterable(chunk)
						].sort(compareByIdentifier);
						const entryModules = [
							...compilation.chunkGraph.getChunkEntryModulesIterable(chunk)
						].sort(compareByIdentifier);
						chunkModules[chunk.id] = {
							modules: modules.map(moduleStringify),
							entryModules: entryModules.map(moduleStringify)
						};
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
