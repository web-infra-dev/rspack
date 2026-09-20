import { defineConfig, definePlugin } from '@rspack/cli';
import { SubresourceIntegrityPlugin } from '@rspack/core';
import fs from 'node:fs';
import path from 'node:path';

export default defineConfig((_, { testPath }) => ({
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
    definePlugin({
      apply(compiler) {
        compiler.hooks.afterEmit.tap('AfterEmitPlugin', (_compilation) => {
          const content = fs.readFileSync(
            path.resolve(testPath, 'bundle0.js'),
            'utf-8',
          );
          expect(content).toContain('sriHashes');
          expect(content).toContain('sriCssHashes');
        });
      },
    }),
  ],
}));
