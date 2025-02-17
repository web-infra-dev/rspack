const expect = require("expect");
const { createIntegrityPlugin, createHtmlPlugin, getDist } = require("../wsi-test-helper");

module.exports = {
  mode: "production",
  entry: {
    index: "./index.js",
  },
  target: "electron-renderer",
  output: {
    crossOriginLoading: "anonymous",
    path: getDist(__dirname),
  },
  plugins: [
    createIntegrityPlugin({
      hashFuncNames: ["sha256", "sha384"],
      enabled: true,
    }),
    createHtmlPlugin(),
    {
      apply: (compiler) => {
        compiler.hooks.done.tapPromise("wsi-test", async (stats) => {
          expect(stats.compilation.warnings.length).toEqual(0);
          expect(stats.compilation.errors.length).toEqual(0);
        });
      },
    },
  ],
};
