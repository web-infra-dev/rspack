const { CssExtractRspackPlugin, SubresourceIntegrityPlugin } = require("@rspack/core");
const fs = require("fs");
const path = require("path");

/** @type {import("@rspack/core").Configuration} */
module.exports = (_, { testPath }) => ({
  target: "web",
  output: {
    crossOriginLoading: "anonymous",
  },
  experiments: {
    css: true
  },
  plugins: [
    new CssExtractRspackPlugin(),
    new SubresourceIntegrityPlugin({
      hashFuncNames: ["sha256", "sha384"],
    }),
    {
      apply(compiler) {
        compiler.hooks.afterEmit.tap("AfterEmitPlugin", (compilation) => {
          const content = fs.readFileSync(path.resolve(testPath, "bundle0.js"), "utf-8");
          expect(content).toContain("sriHashes");
          expect(content).toContain("sriCssHashes");
          expect(content).toContain("sriExtractCssHashes");
        });
      },
    }
  ],
  module: {
    rules: [
      {
        test: /-extract\.module\.css$/,
        use: [
          CssExtractRspackPlugin.loader,
          {
            loader: "css-loader",
            options: {
              modules: {
                auto: true,
              },
            },
          },
        ],
        type: "javascript/auto",
      },
    ],
  },
});