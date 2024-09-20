const { RuntimeModule, RuntimeGlobals } = require("@rspack/core");

class MockRuntimeModule extends RuntimeModule {
  constructor(chunk) {
    super("mock");
    this.chunk = chunk;
  }

  generate(compilation) {
    const chunkIdToName = this.chunk.getChunkMaps(false).name;
    const chunkNameToId = Object.fromEntries(
      Object.entries(chunkIdToName).map(([chunkId, chunkName]) => [
        chunkName,
        chunkId,
      ]),
    );

    return `
      __webpack_require__.mock = function(chunkId) {
        chunkId = (${JSON.stringify(
      chunkNameToId,
    )})[chunkId]||chunkId;
        return ${RuntimeGlobals.publicPath} + ${RuntimeGlobals.getChunkScriptFilename}(chunkId);
      };
    `;
  }
}

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
      compiler.hooks.thisCompilation.tap(
        "MockRuntimePlugin",
        (compilation) => {
          compilation.hooks.runtimeRequirementInTree.tap("MockRuntimePlugin", (chunk, set) => {
            set.add(RuntimeGlobals.publicPath);
            set.add(RuntimeGlobals.getChunkScriptFilename);
            compilation.addRuntimeModule(
              chunk,
              new MockRuntimeModule(chunk)
            );
          })
        }
      );
    }
  ],
};