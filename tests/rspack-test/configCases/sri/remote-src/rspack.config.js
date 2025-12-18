const { SubresourceIntegrityPlugin, HtmlRspackPlugin } = require("@rspack/core");
const HtmlWebpackPlugin = require("html-webpack-plugin");
const fs = require("fs");
const path = require("path");

/** @type {import("@rspack/core").Configuration} */
module.exports = (_, { testPath }) => ([{
  target: "web",
  output: {
    crossOriginLoading: "anonymous",
  },
  plugins: [
    new SubresourceIntegrityPlugin(),
    new HtmlRspackPlugin({
      filename: "index.html",
    }),
    {
      apply(compiler) {
        compiler.hooks.compilation.tap('TestPlugin', (compilation) => {
          HtmlRspackPlugin.getCompilationHooks(compilation).beforeAssetTagGeneration.tap('SubresourceIntegrityPlugin', (data) => {
            data.assets.js.push("http://localhost:3000/index.js");
          });
        });
      }
    },
    {
      apply(compiler) {
        compiler.hooks.done.tap('TestPlugin', () => {
          const htmlContent = fs.readFileSync(path.resolve(testPath, "index.html"), "utf-8");
          expect(htmlContent).toMatch(/<script crossorigin defer integrity=".+" src="bundle0\.js">/);
          expect(htmlContent).toMatch(/<script defer src="http:\/\/localhost:3000\/index\.js">/);
        });
      }
    }
  ],
}, {
  target: "web",
  output: {
    crossOriginLoading: "anonymous",
  },
  plugins: [
    new SubresourceIntegrityPlugin({
      htmlPlugin: require.resolve("html-webpack-plugin"),
    }),
    new HtmlWebpackPlugin({
      filename: "index1.html",
    }),
    {
      apply(compiler) {
        compiler.hooks.compilation.tap('TestPlugin', (compilation) => {
          HtmlWebpackPlugin.getCompilationHooks(compilation).beforeAssetTagGeneration.tap('SubresourceIntegrityPlugin', (data) => {
            data.assets.js.push("http://localhost:3000/index.js");
          });
        });
      }
    },
    {
      apply(compiler) {
        compiler.hooks.done.tap('TestPlugin', () => {
          const htmlContent = fs.readFileSync(path.resolve(testPath, "index1.html"), "utf-8");
          expect(htmlContent).toMatch(/<script defer="defer" src="bundle1.js" integrity=".+" crossorigin="anonymous">/);
          expect(htmlContent).toMatch(/<script defer="defer" src="http:\/\/localhost:3000\/index\.js">/);
        });
      }
    }
  ],
}]);