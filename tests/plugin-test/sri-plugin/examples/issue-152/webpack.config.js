const expect = require("expect");
const { createIntegrityPlugin, getDist } = require("../wsi-test-helper");
module.exports = {
  entry: "./index.js",
  output: {
    filename: "[contenthash].js",
    chunkFilename: "[contenthash].chunk.js",
    crossOriginLoading: "anonymous",
    path: getDist(__dirname),
  },
  devtool: false,
  optimization: {
    moduleIds: "deterministic",
    realContentHash: true,
    chunkIds: "deterministic",
    runtimeChunk: "single",
  },
  plugins: [
    createIntegrityPlugin({
      hashFuncNames: ["sha256"],
      enabled: true,
    }),
    {
      apply: (compiler) => {
        compiler.hooks.done.tap("wsi-test", (stats) => {
          expect(Object.keys(stats.compilation.assets)).toContain(
            "85e3f6d0198e353b.js"
          );
        });
      },
    },
  ],
};
