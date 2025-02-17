const { createIntegrityPlugin, getDist } = require("../wsi-test-helper");
const expect = require("expect");
const ChunkRenderError = require("webpack/lib/ChunkRenderError");

module.exports = {
  mode: "production",
  entry: "./index.js",
  output: {
    filename:
      "[name]-[hash]-[chunkhash]-[hash:4]-[chunkhash:4]-[id]-[query].js",
    crossOriginLoading: "anonymous",
    path: getDist(__dirname),
  },
  plugins: [
    createIntegrityPlugin({ hashFuncNames: ["sha256", "sha384"] }),
    {
      apply: (compiler) => {
        compiler.hooks.thisCompilation.tap("wsi-test", (compilation) => {
          compilation.hooks.renderManifest.tap("wsi-test", () => {
            throw new Error("Provoke ChunkRenderError");
          });
        });
        compiler.hooks.done.tap("wsi-test", (stats) => {
          expect(stats.compilation.warnings.length).toEqual(0);
          expect(stats.compilation.errors.length).toEqual(1);
          expect(stats.compilation.errors[0]).toBeInstanceOf(ChunkRenderError);

          stats.compilation.errors = [];
        });
      },
    },
  ],
};
