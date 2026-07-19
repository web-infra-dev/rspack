const { experiments } = require('@rspack/core');

/** @type {import('@rspack/core').Configuration} */
module.exports = {
  mode: 'production',
  devtool: 'source-map',
  output: {
    publicPath: '/assets/',
  },
  plugins: [
    new experiments.ExtractInlineDataUrlPlugin({
      filename: 'inline/[contenthash:10][ext]',
      minSize: 0,
    }),
    {
      apply(compiler) {
        compiler.hooks.done.tap('VerifyExtractedInlineDataUrl', (stats) => {
          const assets = stats.compilation.getAssets();
          const extracted = assets.find((asset) =>
            /^inline\/.+\.png$/.test(asset.name),
          );
          const main = assets.find(
            (asset) =>
              asset.name.endsWith('.js') && !asset.name.startsWith('inline/'),
          );

          expect(extracted).toBeDefined();
          expect(extracted.source.source().toString()).toBe(
            'hello inline asset',
          );
          expect(main).toBeDefined();
          expect(main.source.source().toString()).toContain(
            `/assets/${extracted.name}`,
          );
          expect(main.source.source().toString()).not.toContain(
            'aGVsbG8gaW5saW5lIGFzc2V0',
          );
          expect(
            assets.some((asset) => asset.name === `${main.name}.map`),
          ).toBe(true);
        });
      },
    },
  ],
};
