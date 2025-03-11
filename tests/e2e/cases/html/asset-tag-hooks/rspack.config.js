const {rspack} = require("@rspack/core");

/** @type { import('@rspack/core').RspackOptions } */
module.exports = {
  context: __dirname,
  mode: "development",
  entry: "./src/index.js",
  stats: "none",
  plugins: [
    new rspack.HtmlRspackPlugin({
      template: "./src/index.html"
    }),
    {
      /** @param {import('@rspack/core').Compiler} compiler */
      apply(compiler) {
        compiler.hooks.compilation.tap("TestPlugin", (compilation) => {
          rspack.HtmlRspackPlugin.getCompilationHooks(compilation).alterAssetTags.tap("TestPlugin", (data) => {
            data.assetTags.scripts.push({
              tagName: 'script',
              innerHTML: 'console.log("injected source code");',
              voidTag: false,
              attributes: {id: 'inner-html-tag'},
            })
            return data;
          });
        });
      }
    }
  ],
  devServer: {
    port: 3000
  }
};
