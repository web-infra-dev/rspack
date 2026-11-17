import path from 'node:path';
import assert from 'node:assert/strict';
import { NormalModule } from '@rspack/core';

export default {
  resolve: {
    alias: {
      'strict-target': path.join(import.meta.dirname, 'target.custom'),
      'strict-ignored': false,
    },
    byDependency: {
      esm: {
        alias: {
          'strict-esm': path.join(import.meta.dirname, 'target.custom'),
        },
      },
    },
  },
  plugins: [{ apply(compiler) {
    compiler.hooks.compilation.tap('StrictMpmc', compilation => {
      NormalModule.getCompilationHooks(compilation).loader.tap('StrictMpmc', context => {
        assert.ok(!context.loaders[context.loaderIndex]?.parallel);
        context.fromHook = true;
      });
      compilation.hooks.processAssets.tap('StrictMpmc', () => {
        if (!compilation.errors.length) assert.equal(compilation.getAsset('strict-resolve.txt').source.source().toString(), 'worker asset');
      });
    });
  } }],
  module: {
    rules: [
      {
        test: /value\.js$/,
        use: [{ loader: './main-loader.js', options: {} }, { loader: './bridge-loader.js', parallel: true, options: { callback: () => 'main API' } }, { loader: './loader.js', parallel: 'strict', options: {} }, { loader: './callback-loader.js', parallel: 'strict', options: {} }, { loader: './main-loader.js', options: { seed: true } }],
      },
    ],
  },
};
