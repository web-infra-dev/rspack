import path from 'node:path';
import { rspack } from '@rspack/core';

/**
 * @type {import("@rspack/core").Configuration}
 */
export default {
  node: {
    __dirname: false,
    __filename: false,
  },
  output: {
    filename: '[name].js',
  },
  plugins: [
    new rspack.SourceMapDevToolPlugin({
      test: /\.js/,
      filename: '[file].map',
      sourceRoot: path.join(import.meta.dirname, 'folder') + '/',
    }),
    new rspack.DefinePlugin({
      CONTEXT: JSON.stringify(import.meta.dirname),
    }),
    (compiler) => {
      compiler.hooks.compilation.tap('PLUGIN', (compilation) => {
        compilation.hooks.afterProcessAssets.tap('PLUGIN', (assets) => {
          for (const asset of Object.values(assets)) {
            expect(typeof asset.source()).toBe('string');
          }
        });
      });
    },
  ],
};
