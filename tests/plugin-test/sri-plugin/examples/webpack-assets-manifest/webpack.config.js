const WebpackAssetsManifest = require("webpack-assets-manifest");
const expect = require("expect");
const { readFileSync } = require("fs");
const { join } = require("path");
const { createIntegrityPlugin, getDist } = require("../wsi-test-helper");
module.exports = {
  entry: {
    index: "./index.js",
  },
  output: {
    crossOriginLoading: "anonymous",
    path: getDist(__dirname),
  },
  plugins: [
    createIntegrityPlugin({
      hashFuncNames: ["sha384", "sha512"],
      enabled: true,
    }),
    new WebpackAssetsManifest({ integrity: true }),
    {
      apply: (compiler) => {
        compiler.hooks.done.tap("wsi-test", () => {
          const manifest = JSON.parse(
            readFileSync(join(getDist(__dirname), "manifest.json"), "utf-8")
          );
          expect(manifest["index.js"].integrity).toMatch(/sha384-.* sha512-.*/);
        });
      },
    },
  ],
};
