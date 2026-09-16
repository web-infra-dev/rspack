import { rspack } from '@rspack/core';
import path from 'node:path';
import assert from 'node:assert';

/** @type {import("@rspack/core").Configuration} */
export default {
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.make.tap('child', (compilation) => {
          const childCompiler = compilation.createChildCompiler(
            'child',
            {
              filename: 'child.js',
            },
            [
              new compiler.rspack.EntryPlugin(
                compiler.context,
                path.resolve(import.meta.dirname, './child.js'),
                { name: 'child' },
              ),
            ],
          );
          childCompiler.compile((_err, result) => {
            const assets = result
              .getAssets()
              .filter((asset) => asset.name === 'child.js');
            assert(assets.length === 1);
            const asset = assets[0];
            assert(asset.source.source().toString().includes('hello/1'));
          });
        });
      },
    },
  ],
  optimization: {
    minimize: true,
    minimizer: [new rspack.SwcJsMinimizerRspackPlugin()],
  },
};
