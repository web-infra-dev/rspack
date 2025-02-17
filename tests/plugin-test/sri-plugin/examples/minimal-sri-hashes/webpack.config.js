const { createHtmlPlugin, createIntegrityPlugin, getDist } = require("../wsi-test-helper");
const { readFileSync } = require("fs");
const { join } = require("path");
const expect = require("expect");

module.exports = {
  mode: "development",
  entry: {
    mainAppChunk: ["./index.js"],
  },
  output: {
    filename: "[name].js",
    publicPath: "/",
    crossOriginLoading: "anonymous",
    path: getDist(__dirname),
  },
  optimization: {
    runtimeChunk: "single",
    splitChunks: {
      chunks: "all",
      cacheGroups: {
        vendors: {
          test: /node_modules/,
          name: "vendors",
          chunks: "all",
        },
      },
    },
  },
  plugins: [
    createHtmlPlugin(),
    createIntegrityPlugin({
      hashFuncNames: ["sha256", "sha384"],
    }),
    {
      apply: (compiler) => {
        compiler.hooks.done.tap("wsi-test", () => {
          const runtimeJs = readFileSync(
            join(getDist(__dirname), "runtime.js"),
            "utf-8"
          );
          expect(runtimeJs).not.toMatch(/mainAppChunk/);
        });
      },
    },
  ],
};
