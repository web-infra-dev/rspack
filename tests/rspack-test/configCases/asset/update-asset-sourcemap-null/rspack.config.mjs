import assert from 'node:assert';
import fs from 'node:fs';

/**
 * @type {import('@rspack/core').Configuration}
 */
export default {
  context: import.meta.dirname,
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.compilation.tap('PLUGIN', (compilation) => {
          compilation.hooks.processAssets.tap('PLUGIN', (assets) => {
            for (const name in assets) {
              const source = assets[name];
              compilation.updateAsset(name, source, {
                related: { sourceMap: null },
              });
            }
          });
        });
      },
    },
  ],
};
