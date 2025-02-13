const MiniCssExtractPlugin = require("mini-css-extract-plugin");
const WebpackAssetsManifest = require("webpack-assets-manifest");
const FixStyleOnlyEntriesPlugin = require("webpack-fix-style-only-entries");
const expect = require("expect");
const { createIntegrityPlugin, getDist } = require("../wsi-test-helper");

module.exports = {
  mode: "production",
  entry: {
    index: "./index.js",
    style: ["./style.css"],
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        use: [
          MiniCssExtractPlugin.loader,
          {
            loader: "css-loader",
            options: {
              importLoaders: 1,
            },
          },
        ],
      },
    ],
  },
  output: {
    crossOriginLoading: "anonymous",
  },
  plugins: [
    new FixStyleOnlyEntriesPlugin({
      silent: true,
    }),
    new MiniCssExtractPlugin({
      filename: "[name].css",
    }),
    new WebpackAssetsManifest({ integrity: true }),
    createIntegrityPlugin({
      hashFuncNames: ["sha256", "sha384"],
      enabled: true,
    }),
    {
      apply: (compiler) => {
        compiler.hooks.done.tap("wsi-test", (stats) => {
          expect(
            stats.compilation.warnings.filter(
              // Ignore Webpack deprecation warnings
              (message) => {
                console.log(message);
                return !message.match(/DEP_WEBPACK_/);
              }
            ).length
          ).toEqual(0);
        });
      },
    },
  ],
};
