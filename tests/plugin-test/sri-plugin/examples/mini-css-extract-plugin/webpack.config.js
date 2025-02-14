const { createHtmlPlugin, createIntegrityPlugin, getDist } = require("../wsi-test-helper");
const { CssExtractRspackPlugin } = require("@rspack/core");
const { RunInPuppeteerPlugin } = require("../wsi-test-helper");

module.exports = {
  entry: {
    index: "./index.js",
  },
  plugins: [
    new CssExtractRspackPlugin({
      // Options similar to the same options in webpackOptions.output
      // both options are optional
      filename: "[name].css",
      chunkFilename: "[id].css",
    }),
    createIntegrityPlugin({
      hashFuncNames: ["sha256", "sha384"],
      enabled: true,
    }),
    createHtmlPlugin(),
    new RunInPuppeteerPlugin(),
  ],
  devtool: false,
  output: {
    crossOriginLoading: "anonymous",
    path: getDist(__dirname),
  },
  optimization: {
    minimize: false
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        use: [
          CssExtractRspackPlugin.loader,
          {
            loader: "css-loader",
            options: {
              sourceMap: true,
              modules: {
                auto: true,
              },
              importLoaders: 1,
            },
          },
        ],
      },
    ],
  },
};
