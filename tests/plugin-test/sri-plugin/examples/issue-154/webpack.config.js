const { createHtmlPlugin, createIntegrityPlugin, getDist } = require("../wsi-test-helper");
const { CssExtractRspackPlugin } = require("@rspack/core");
const expect = require("expect");
const { RunInPuppeteerPlugin } = require("../wsi-test-helper");

module.exports = {
  // mode: "development",
  devtool: "cheap-module-source-map",
  entry: "./index.js",
  output: {
    filename: "[contenthash].js",
    chunkFilename: "[contenthash].chunk.js",
    crossOriginLoading: "anonymous",
    path: getDist(__dirname),
  },
  optimization: {
    moduleIds: "deterministic",
    realContentHash: true,
    chunkIds: "deterministic",
    runtimeChunk: "single",
  },
  module: {
    rules: [
      {
        test: /\.css$/i,
        use: [CssExtractRspackPlugin.loader, "css-loader"],
      },
    ],
  },
  plugins: [
    new CssExtractRspackPlugin({
      filename: `[contenthash].css`,
      chunkFilename: `[contenthash].chunk.css`,
    }),
    createIntegrityPlugin({
      enabled: true,
    }),
    createHtmlPlugin(),
    {
      apply: (compiler) => {
        compiler.hooks.done.tap("wsi-test", (stats) => {
          const cssAsset = stats
            .toJson()
            .assets.find((asset) => asset.name.match(/\.css$/));

          expect(cssAsset.info.contenthash).toBeDefined();
          expect(
            cssAsset.info.contenthash.find((hash) => hash.match(/^sha/))
          ).toBeDefined();
          expect(cssAsset.integrity).toBeDefined();
        });
      },
    },
    new RunInPuppeteerPlugin(),
  ],
};
