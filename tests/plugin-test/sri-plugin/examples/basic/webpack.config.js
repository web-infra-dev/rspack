const expect = require("expect");
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
      hashFuncNames: ["sha256"],
      enabled: true,
    }),
    {
      apply: (compiler) => {
        compiler.hooks.done.tap("wsi-test", (stats) => {
          expect(
            !stats.toJson().assets.find((asset) => asset.name == "index.js")
              .integrity
          ).not.toBeNull();
        });
      },
    },
  ],
};
