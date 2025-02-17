const { createIntegrityPlugin, createHtmlPlugin, RunInPuppeteerPlugin, getDist } = require("../wsi-test-helper");

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
    createHtmlPlugin(),
    new RunInPuppeteerPlugin(),
  ],
};
