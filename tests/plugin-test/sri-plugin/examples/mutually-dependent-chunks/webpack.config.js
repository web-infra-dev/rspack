const expect = require("expect");
const { createIntegrityPlugin, getDist } = require("../wsi-test-helper");

module.exports = {
  mode: "production",
  entry: "./main.js",
  output: {
    filename: "bundle.js",
    crossOriginLoading: "anonymous",
    path: getDist(__dirname),
  },
  plugins: [
    createIntegrityPlugin({ hashFuncNames: ["sha256", "sha384"] }),
    {
      apply: (compiler) => {
        compiler.hooks.done.tap("wsi-test", (stats) => {
          expect(stats.hasWarnings()).toBeFalsy();
          stats.toJson().assets.forEach((asset) => {
            expect(asset.integrity).toMatch(/^sha/);
          });
        });
      },
    },
  ],
};
