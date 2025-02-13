const { createHtmlPlugin, createIntegrityPlugin, getDist } = require("../wsi-test-helper");
const { RunInPuppeteerPlugin } = require("../wsi-test-helper");

module.exports = {
  entry: "./index.js",
  mode: "production",
  output: {
    filename: "[name].[contenthash].js",
    chunkFilename: "[name].[contenthash].js",
    crossOriginLoading: "anonymous",
    path: getDist(__dirname),
  },
  plugins: [
    createHtmlPlugin(),
    createIntegrityPlugin({
      hashFuncNames: ["sha256", "sha384"],
    }),
    new RunInPuppeteerPlugin(),
  ],
};
