const { HotModuleReplacementPlugin } = require("@rspack/core");
const { createIntegrityPlugin, getDist } = require("../wsi-test-helper");
const expect = require("expect");

module.exports = {
  mode: "production",
  entry: "./index.js",
  output: {
    filename: "bundle.js",
    crossOriginLoading: "anonymous",
    path: getDist(__dirname),
  },
  plugins: [
    new HotModuleReplacementPlugin(),
    createIntegrityPlugin({ hashFuncNames: ["sha256", "sha384"] }),
    {
      apply: (compiler) => {
        compiler.hooks.done.tap("wsi-test", (stats) => {
          expect(stats.compilation.warnings.length).toEqual(1);
          expect(stats.compilation.warnings[0]).toHaveProperty("message");
          expect(stats.compilation.warnings[0].message).toMatch(
            /may interfere with hot reloading./
          );
        });
      },
    },
  ],
};
