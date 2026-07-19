const { experiments } = require('@rspack/core');

/** @type {import('@rspack/core').Configuration} */
module.exports = {
  mode: 'production',
  devtool: 'source-map',
  optimization: {
    minimize: false,
  },
  plugins: [
    new experiments.ManglePrivateIdentifiersPlugin({
      reserved: ['_reservedPrivateValue'],
    }),
    {
      apply(compiler) {
        compiler.hooks.done.tap('VerifyMangledPrivateIdentifiers', (stats) => {
          const assets = stats.compilation.getAssets();
          const javascript = assets
            .filter((asset) => asset.name.endsWith('.js'))
            .map((asset) => asset.source.source().toString())
            .join('\n');

          expect(javascript).not.toContain('_longPrivateValue');
          expect(javascript).not.toContain('_longPrivateMethod');
          expect(javascript).toContain('_reservedPrivateValue');
          expect(javascript).toContain('_reflectedPrivateValue');
          expect(
            assets.some(
              (asset) =>
                asset.name.endsWith('.js') &&
                assets.some((item) => item.name === `${asset.name}.map`),
            ),
          ).toBe(true);
        });
      },
    },
  ],
};
