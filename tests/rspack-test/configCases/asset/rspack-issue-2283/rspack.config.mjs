import assert from 'node:assert';
import fs from 'node:fs';

/**
 * @type {import('@rspack/core').Configuration}
 */
export default {
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: /\.png$/,
        type: 'asset/resource',
      },
    ],
  },
  plugins: [
    new (class {
      apply(compiler) {
        compiler.hooks.compilation.tap('MyPlugin', (compilation) => {
          compilation.hooks.processAssets.tap('MyPlugin', (assets) => {
            let list = Object.keys(assets);
            const png = list.find((item) => item.endsWith('png'));
            const asset = compilation.getAsset(png);
            const buf = asset.source.buffer();
            const expected = fs.readFileSync(
              import.meta.dirname + '/' + 'img.png',
            );
            assert.deepEqual(buf, expected);
          });
        });
      }
    })(),
  ],
};
