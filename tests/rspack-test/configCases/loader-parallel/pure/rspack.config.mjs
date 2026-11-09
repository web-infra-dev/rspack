import assert from 'node:assert/strict';
import { NormalModule } from '@rspack/core';

export default {
  output: { hashFunction: 'sha256' },
  loader: { pureFixture: 'worker option' },
  module: {
    rules: [
      {
        test: /value\.js$/,
        use: [
          { loader: './main-loader.js', options: { callback: () => 'main' } },
          { loader: './worker-loader.js', parallel: true, options: {} },
          {
            loader: './pure-loader.js',
            parallel: 'pure',
            options: { suffix: 'pure' },
          },
        ],
      },
    ],
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.compilation.tap('PureLoader', (compilation) => {
          compilation.hooks.processAssets.tap('PureLoader', () => {
            if (compilation.errors.length > 0) return;
            const asset = compilation.getAsset('pure.txt');
            assert.ok(asset);
            assert.equal(asset.source.source().toString(), 'worker asset');
            assert.equal(asset.info.immutable, true);
          });
          NormalModule.getCompilationHooks(compilation).loader.tap(
            'PureLoader',
            (context) => {
              if (context.loaders[context.loaderIndex]?.parallel === 'pure') {
                throw new Error('Pure loader reached the loader hook');
              }
              context.fromLoaderHook = true;
            },
          );
        });
      },
    },
  ],
};
