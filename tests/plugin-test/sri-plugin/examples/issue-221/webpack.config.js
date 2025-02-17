const { createHtmlPlugin, createIntegrityPlugin, getDist } = require("../wsi-test-helper");
const expect = require("expect");

module.exports = {
  entry: {
    "why 1+1=2?": "./index.js",
  },
  output: {
    crossOriginLoading: "anonymous",
    path: getDist(__dirname),
  },
  plugins: [
    createIntegrityPlugin({
      hashFuncNames: ["sha256"],
      enabled: true,
    }),
    createHtmlPlugin(),
    {
      apply: (compiler) => {
        compiler.hooks.done.tap("wsi-test", (stats) => {
          expect(
            stats.toJson().assets.find((asset) => asset.name == "why 1+1=2?.js")
          ).toHaveProperty("integrity");
        });
      },
    },
  ],
};
