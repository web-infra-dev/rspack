import {
  CssExtractRspackPlugin,
  SubresourceIntegrityPlugin,
} from '@rspack/core';
import fs from 'node:fs';
import path from 'node:path';

/** @type {import("@rspack/core").Configuration} */
export default (_, { testPath }) => ({
  target: 'web',
  output: {
    crossOriginLoading: 'anonymous',
  },
  experiments: {
    css: false,
  },
  plugins: [
    new CssExtractRspackPlugin(),
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
          expect(content).toContain('sriExtractCssHashes');
        });
      },
    },
  ],
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'javascript/auto',
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
      },
    ],
  },
});
