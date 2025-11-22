const { experiments, HtmlRspackPlugin } = require("@rspack/core");
const HtmlWebpackPlugin = require("html-webpack-plugin");
const fs = require("fs");
const path = require("path");

/** @type {import("@rspack/core").Configuration} */
module.exports = (_, { testPath }) => ([{
  target: "web",
  output: {
    publicPath: "http://localhost:3000/",
    chunkFilename: "[name].0.js",
    crossOriginLoading: "anonymous",
  },
  plugins: [
    new experiments.SubresourceIntegrityPlugin(),
    new HtmlRspackPlugin({
      filename: "index.html",
    }),
    {
      apply(compiler) {
        compiler.hooks.compilation.tap('TestPlugin', (compilation) => {
          HtmlRspackPlugin.getCompilationHooks(compilation).beforeAssetTagGeneration.tap('SubresourceIntegrityPlugin', (data) => {
            data.assets.js.push("//localhost:3000/chunk.0.js");
            data.assets.js.push("http://localhost:3000/chunk.0.js");
            data.assets.js.push("//rspack.dev/chunk.0.js");
            data.assets.js.push("http://rspack.dev/chunk.0.js");
          });
        });
      }
    },
    {
      apply(compiler) {
        compiler.hooks.done.tap('TestPlugin', () => {
          const htmlContent = fs.readFileSync(path.resolve(testPath, "index.html"), "utf-8");
          expect(htmlContent).toMatch(/<script crossorigin defer integrity=".+" src="\/\/localhost:3000\/chunk\.0\.js">/);
          expect(htmlContent).toMatch(/<script crossorigin defer integrity=".+" src="http:\/\/localhost:3000\/chunk\.0\.js">/);
          expect(htmlContent).toMatch(/<script defer src="\/\/rspack.dev\/chunk\.0\.js">/);
          expect(htmlContent).toMatch(/<script defer src="http:\/\/rspack.dev\/chunk\.0\.js">/);
        });
      }
    }
  ],
}, {
  target: "web",
  output: {
    publicPath: "http://localhost:3000/",
    chunkFilename: "[name].1.js",
    crossOriginLoading: "anonymous",
  },
  plugins: [
    new experiments.SubresourceIntegrityPlugin({
      htmlPlugin: require.resolve("html-webpack-plugin"),
    }),
    new HtmlWebpackPlugin({
      filename: "index1.html",
    }),
    {
      apply(compiler) {
        compiler.hooks.compilation.tap('TestPlugin', (compilation) => {
          HtmlWebpackPlugin.getCompilationHooks(compilation).beforeAssetTagGeneration.tap('SubresourceIntegrityPlugin', (data) => {
            data.assets.js.push("//localhost:3000/chunk.1.js");
            data.assets.js.push("http://localhost:3000/chunk.1.js");
            data.assets.js.push("//rspack.dev/chunk.1.js");
            data.assets.js.push("http://rspack.dev/chunk.1.js");
          });
        });
      }
    },
    {
      apply(compiler) {
        compiler.hooks.done.tap('TestPlugin', () => {
          const htmlContent = fs.readFileSync(path.resolve(testPath, "index1.html"), "utf-8");
          expect(htmlContent).toMatch(/<script defer="defer" src="\/\/localhost:3000\/chunk\.1\.js" integrity=".+" crossorigin="anonymous">/);
          expect(htmlContent).toMatch(/<script defer="defer" src="http:\/\/localhost:3000\/chunk\.1\.js" integrity=".+" crossorigin="anonymous">/);
          expect(htmlContent).toMatch(/<script defer="defer" src="\/\/rspack.dev\/chunk\.1\.js">/);
          expect(htmlContent).toMatch(/<script defer="defer" src="http:\/\/rspack.dev\/chunk\.1\.js">/);
        });
      }
    }
  ],
}, {
  target: "web",
  output: {
    publicPath: "//localhost:3000/",
    chunkFilename: "[name].2.js",
    crossOriginLoading: "anonymous",
  },
  plugins: [
    new experiments.SubresourceIntegrityPlugin(),
    new HtmlRspackPlugin({
      filename: "index2.html",
    }),
    {
      apply(compiler) {
        compiler.hooks.compilation.tap('TestPlugin', (compilation) => {
          HtmlRspackPlugin.getCompilationHooks(compilation).beforeAssetTagGeneration.tap('SubresourceIntegrityPlugin', (data) => {
            data.assets.js.push("//localhost:3000/chunk.2.js");
            data.assets.js.push("http://localhost:3000/chunk.2.js");
            data.assets.js.push("//rspack.dev/chunk.2.js");
            data.assets.js.push("http://rspack.dev/chunk.2.js");
          });
        });
      }
    },
    {
      apply(compiler) {
        compiler.hooks.done.tap('TestPlugin', () => {
          const htmlContent = fs.readFileSync(path.resolve(testPath, "index2.html"), "utf-8");
          expect(htmlContent).toMatch(/<script crossorigin defer integrity=".+" src="\/\/localhost:3000\/chunk\.2\.js">/);
          expect(htmlContent).toMatch(/<script crossorigin defer integrity=".+" src="http:\/\/localhost:3000\/chunk\.2\.js">/);
          expect(htmlContent).toMatch(/<script defer src="\/\/rspack.dev\/chunk\.2\.js">/);
          expect(htmlContent).toMatch(/<script defer src="http:\/\/rspack.dev\/chunk\.2\.js">/);
        });
      }
    }
  ],
}, {
  target: "web",
  output: {
    publicPath: "//localhost:3000/",
    chunkFilename: "[name].3.js",
    crossOriginLoading: "anonymous",
  },
  plugins: [
    new experiments.SubresourceIntegrityPlugin({
      htmlPlugin: require.resolve("html-webpack-plugin"),
    }),
    new HtmlWebpackPlugin({
      filename: "index3.html",
    }),
    {
      apply(compiler) {
        compiler.hooks.compilation.tap('TestPlugin', (compilation) => {
          HtmlWebpackPlugin.getCompilationHooks(compilation).beforeAssetTagGeneration.tap('SubresourceIntegrityPlugin', (data) => {
            data.assets.js.push("//localhost:3000/chunk.3.js");
            data.assets.js.push("http://localhost:3000/chunk.3.js");
            data.assets.js.push("//rspack.dev/chunk.3.js");
            data.assets.js.push("http://rspack.dev/chunk.3.js");
          });
        });
      }
    },
    {
      apply(compiler) {
        compiler.hooks.done.tap('TestPlugin', () => {
          const htmlContent = fs.readFileSync(path.resolve(testPath, "index3.html"), "utf-8");
          expect(htmlContent).toMatch(/<script defer="defer" src="\/\/localhost:3000\/chunk\.3\.js" integrity=".+" crossorigin="anonymous">/);
          expect(htmlContent).toMatch(/<script defer="defer" src="http:\/\/localhost:3000\/chunk\.3\.js" integrity=".+" crossorigin="anonymous">/);
          expect(htmlContent).toMatch(/<script defer="defer" src="\/\/rspack.dev\/chunk\.3\.js">/);
          expect(htmlContent).toMatch(/<script defer="defer" src="http:\/\/rspack.dev\/chunk\.3\.js">/);
        });
      }
    }
  ],
}]);