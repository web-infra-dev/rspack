let firstAsset;

class CheckAssetPlugin {
  apply(compiler) {
    compiler.hooks.thisCompilation.tap('CheckAssetPlugin', (compilation) => {
      compilation.hooks.afterProcessAssets.tap('CheckAssetPlugin', (assets) => {
        const asset = Object.keys(assets).find(
          (name) => name.startsWith('main.') && name.endsWith('.mjs'),
        );
        if (!asset) {
          throw new Error('JavaScript asset not found');
        }
        if (firstAsset === undefined) {
          firstAsset = asset;
        } else {
          expect(asset).not.toBe(firstAsset);
        }
      });
    });
  }
}

const createConfig = () => ({
  mode: 'production',
  target: 'async-node',
  entry: './index.js',
  experiments: {
    fasterModuleConcatenation: true,
  },
  output: {
    filename: '[name].[contenthash].mjs',
    module: true,
    library: {
      type: 'modern-module',
    },
  },
  optimization: {
    minimize: false,
    runtimeChunk: false,
  },
  plugins: [new CheckAssetPlugin()],
});

module.exports = [createConfig(), createConfig()];
