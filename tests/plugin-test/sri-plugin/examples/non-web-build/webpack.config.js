const { createHtmlPlugin, createIntegrityPlugin, getDist } = require("../wsi-test-helper");
const expect = require("expect");

module.exports = {
  mode: "production",
  entry: {
    index: "./index.js",
  },
  target: "node",
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
        compiler.hooks.done.tap("wsi-test", (stats) => {
          expect(stats.compilation.errors.length).toEqual(0);
          expect(stats.compilation.warnings.length).toEqual(1);
          expect(stats.compilation.warnings[0].message).toMatch(
            /This plugin is not useful for non-web targets/
          );
        });
      },
    },
  ],
};
