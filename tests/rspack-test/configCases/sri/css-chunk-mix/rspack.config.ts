import { defineConfig, definePlugin } from '@rspack/cli';
import {
  CssExtractRspackPlugin,
  SubresourceIntegrityPlugin,
} from '@rspack/core';
import fs from 'node:fs';
import path from 'node:path';

export default defineConfig((_, { testPath }) => ({
  target: 'web',
  output: {
    crossOriginLoading: 'anonymous',
  },
  plugins: [
    new CssExtractRspackPlugin(),
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
          expect(content).toContain('sriExtractCssHashes');
        });
      },
    }),
  ],
  module: {
    rules: [
      {
        test: /\.module\.css/,
        type: 'css/module',
      },
      {
        test: /-extract\.module\.css$/,
        use: [
          CssExtractRspackPlugin.loader,
          {
            loader: 'css-loader',
            options: {
              modules: {
                auto: true,
              },
            },
          },
        ],
        type: 'javascript/auto',
      },
    ],
  },
}));
