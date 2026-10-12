import { defineConfig } from '@rspack/cli';
import type { Configuration } from '@rspack/core';

const common = {
  target: 'node22',
  experiments: { deferImport: true },
  optimization: { sideEffects: true, concatenateModules: false },
} satisfies Configuration;

export default defineConfig([
  ...['import', 'module', 'promise'].flatMap((externalType) =>
    [false, true].map((minimize) => ({
      ...common,
      entry: './index.js',
      output: { importFunctionName: 'testImport' },
      externals: {
        external:
          externalType === 'promise'
            ? "promise Promise.resolve(require('node:path'))"
            : `${externalType} node:path`,
      },
      optimization: { ...common.optimization, minimize },
    })),
  ),
  ...[false, true].map((minimize) => ({
    ...common,
    entry: './deferred.js',
    optimization: { ...common.optimization, minimize },
  })),
]);
