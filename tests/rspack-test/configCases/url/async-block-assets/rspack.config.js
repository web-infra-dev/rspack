module.exports = {
  mode: 'development',
  devtool: false,
  target: 'web',
  output: {
    assetModuleFilename: '[name][ext]',
    publicPath: '/path/',
  },
  module: {
    rules: [
      { test: /resource\.txt$/, dependency: 'url', type: 'asset/resource' },
      { test: /inline\.txt$/, dependency: 'url', type: 'asset/inline' },
      { test: /[/\\]source\.txt$/, dependency: 'url', type: 'asset/source' },
      {
        test: /custom\.txt$/,
        dependency: 'url',
        type: 'asset/resource',
        generator: { publicPath: '/custom/', filename: '[name][ext]' },
      },
    ],
  },
  plugins: [
    (compiler) => {
      compiler.hooks.compilation.tap('CheckUrlAssets', (compilation) => {
        compilation.hooks.finishModules.tap('CheckUrlAssets', (modules) => {
          const issuer = [...modules].find(
            (module) => module.rawRequest === './index.js',
          );
          expect(issuer.blocks).toHaveLength(0);
          expect(
            issuer.dependencies.filter((dep) => dep.type === 'new URL()'),
          ).toHaveLength(4);
        });
        compilation.hooks.processAssets.tap('CheckUrlAssets', () => {
          expect(compilation.getAsset('resource.txt')).toBeDefined();
          expect(compilation.getAsset('custom.txt')).toBeDefined();
          expect(compilation.getAsset('inline.txt')).toBeUndefined();
          expect(compilation.getAsset('source.txt')).toBeUndefined();
        });
      });
    },
  ],
};
