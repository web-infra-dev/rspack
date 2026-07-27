const { rspack } = require('@rspack/core');

// deleting via the assets proxy only drops the source: the asset entry and the
// chunk file stay behind, which is exactly the shape this case covers
class DropCssAssetSourcePlugin {
  constructor() {
    this.builds = 0;
  }
  apply(compiler) {
    compiler.hooks.thisCompilation.tap(
      'DropCssAssetSourcePlugin',
      (compilation) => {
        const isRebuild = this.builds++ > 0;
        if (!isRebuild) return;
        compilation.hooks.processAssets.tap(
          {
            name: 'DropCssAssetSourcePlugin',
            stage: compiler.webpack.Compilation.PROCESS_ASSETS_STAGE_OPTIMIZE,
          },
          (assets) => {
            delete assets['main.css'];
          },
        );
      },
    );
  }
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'javascript/auto',
        use: [rspack.CssExtractRspackPlugin.loader, 'css-loader'],
      },
    ],
  },
  plugins: [
    new rspack.CssExtractRspackPlugin({
      filename: '[name].css',
    }),
    new DropCssAssetSourcePlugin(),
  ],
};
