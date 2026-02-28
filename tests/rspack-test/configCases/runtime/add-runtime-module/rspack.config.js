const { RuntimeModule } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	mode: "development",
	devtool: false,
	optimization: {
		minimize: false,
		sideEffects: false,
		concatenateModules: false,
		usedExports: false,
		innerGraph: false,
		providedExports: false
	},
	plugins: [
		compiler => {
			const RuntimeGlobals = compiler.rspack.RuntimeGlobals;
			class MockRuntimeModule extends RuntimeModule {
				constructor() {
					super("mock");
				}

				generate() {
					const chunkIdToName = this.chunk.getChunkMaps(false).name;
					const chunkNameToId = Object.fromEntries(
						Object.entries(chunkIdToName).map(([chunkId, chunkName]) => [
							chunkName,
							chunkId
						])
					);

					return `
						${RuntimeGlobals.require}.mock = function(chunkId) {
							chunkId = (${JSON.stringify(chunkNameToId)})[chunkId]||chunkId;
							return ${RuntimeGlobals.publicPath} + ${RuntimeGlobals.getChunkScriptFilename}(chunkId);
						};
					`;
				}
			}


			compiler.hooks.thisCompilation.tap("MockRuntimePlugin", compilation => {
				compilation.hooks.runtimeRequirementInTree
					.for(RuntimeGlobals.ensureChunkHandlers)
					.tap("MockRuntimePlugin", (chunk, set) => {
						set.add(RuntimeGlobals.publicPath);
						set.add(RuntimeGlobals.getChunkScriptFilename);
						compilation.addRuntimeModule(chunk, new MockRuntimeModule(chunk));
					});
			});
		}
	]
};
