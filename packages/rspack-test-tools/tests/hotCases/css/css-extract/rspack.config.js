const rspack = require("@rspack/core");
const PLUGIN_NAME = "test plugin";

class Plugin {
  color = ['red', 'blue'];
  apply(compiler) {
    compiler.hooks.thisCompilation.tap(PLUGIN_NAME, (compilation) => {
      compilation.hooks.processAssets.tapPromise({
        name: PLUGIN_NAME,
        stage: 300,
      }, async (assets) => {
        for (const [filename, source] of Object.entries(assets)) {
          if (!filename.endsWith('.css')) {
            continue
          }
          const content = source.source().toString('utf-8')
          expect(content).toContain(this.color.shift());
        }
      })
    });
  }
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  cache: true,
  mode: "development",
  entry: "./index",
  module: {
    rules: [
      {
        test: /\.css$/,
        use: [
          {
            loader: rspack.CssExtractRspackPlugin.loader
          },
          "css-loader",
        ]
      },
    ]
  },
  experiments: {
    css: false,
  },
  plugins: [
    new Plugin(),
    new rspack.CssExtractRspackPlugin({
      filename: "[name].css"
    }),
  ]
};