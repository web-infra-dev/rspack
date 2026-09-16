import { SubresourceIntegrityPlugin } from '@rspack/core';
import fs from 'node:fs';
import path from 'node:path';

/** @type {import("@rspack/core").Configuration} */
export default (_, { testPath }) => ({
  target: 'web',
  output: {
    crossOriginLoading: 'anonymous',
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
  plugins: [
    new SubresourceIntegrityPlugin({
      hashFuncNames: ['sha256', 'sha384'],
    }),
    {
      apply(compiler) {
        compiler.hooks.afterEmit.tap('AfterEmitPlugin', (compilation) => {
          const content = fs.readFileSync(
            path.resolve(testPath, 'bundle0.js'),
            'utf-8',
          );
          expect(content).toContain('sriHashes');
          expect(content).toContain('sriCssHashes');
        });
      },
    },
  ],
});
