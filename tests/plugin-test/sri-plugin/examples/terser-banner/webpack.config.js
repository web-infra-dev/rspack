const { createHtmlPlugin, createIntegrityPlugin, getDist } = require("../wsi-test-helper");
const { RunInPuppeteerPlugin } = require("../wsi-test-helper");
const TerserPlugin = require("terser-webpack-plugin");

module.exports = {
  entry: {
    index: "./index.js",
  },
  output: {
    crossOriginLoading: "anonymous",
    path: getDist(__dirname),
  },
  optimization: {
    minimize: true,
    minimizer: [
      new TerserPlugin({
        extractComments: {
          condition: () => true,
          filename: () => "LICENSE.txt",
          banner: (licenseFile) => {
            return `License information can be found in ${licenseFile}`;
          },
        },
      }),
    ],
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
