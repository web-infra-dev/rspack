const expect = require("expect");
const { createIntegrityPlugin, getDist } = require("../wsi-test-helper");

module.exports = {
  mode: "production",
  entry: "./index.js",
  output: {
    filename: "bundle.js",
    path: getDist(__dirname),
  },
  plugins: [
    createIntegrityPlugin({
      hashFuncNames: ["sha256"],
      enabled: false,
    }),
    {
      apply: (compiler) => {
        compiler.hooks.done.tapPromise("wsi-test", async (stats) => {
          expect(stats.compilation.warnings.length).toEqual(0);
          expect(
            Object.keys(
              stats.toJson().assets.find((asset) => asset.name === "bundle.js")
            )
          ).not.toContain("integrity");
        });
      },
    },
  ],
};
