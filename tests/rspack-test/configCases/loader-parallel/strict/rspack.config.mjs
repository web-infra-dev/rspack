import assert from 'node:assert/strict';
import { NormalModule } from '@rspack/core';

export default {
  output: { hashFunction: 'sha256' },
  loader: { strictFixture: 'worker option' },
  module: {
    rules: [
      {
        test: /value\.js$/,
        use: [
          { loader: './main-loader.js', options: { callback: () => 'main' } },
          { loader: './worker-loader.js', parallel: true, options: {} },
          {
            loader: './strict-loader.js',
            parallel: 'strict',
            options: { suffix: 'strict', consume: true },
          },
          {
            loader: './strict-loader.js',
            parallel: 'strict',
            options: { suffix: 'strict' },
          },
          { loader: './main-loader.js', options: { seed: true } },
        ],
      },
    ],
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.compilation.tap('StrictLoader', (compilation) => {
          compilation.hooks.processAssets.tap('StrictLoader', () => {
            if (compilation.errors.length > 0) return;
            const asset = compilation.getAsset('strict.txt');
            assert.ok(asset);
            assert.equal(asset.source.source().toString(), 'worker asset');
            assert.equal(asset.info.immutable, true);
          });
          NormalModule.getCompilationHooks(compilation).loader.tap(
            'StrictLoader',
            (context) => {
              if (context.loaders[context.loaderIndex]?.parallel === 'strict') {
                throw new Error('Strict loader reached the loader hook');
              }
              if (context.loaders[context.loaderIndex]?.parallel === true) {
                context.getOptions().fromRegularParallelHook = true;
              }
              context.fromLoaderHook = true;
            },
          );
        });
      },
    },
  ],
};
