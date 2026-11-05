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
  experiments: {
    css: false,
  },
  optimization: {
    chunkIds: 'named',
    moduleIds: 'named',
    minimize: false,
    // Move the extracted css of every async chunk into a chunk of its own, so
    // that chunk holds `css/mini-extract` modules and nothing else.
    splitChunks: {
      chunks: 'async',
      cacheGroups: {
        styles: {
          type: 'css/mini-extract',
          name: 'styles',
          chunks: 'async',
          enforce: true,
        },
      },
    },
  },
  plugins: [
    new CssExtractRspackPlugin(),
    new SubresourceIntegrityPlugin(),
    definePlugin({
      apply(compiler) {
        compiler.hooks.afterEmit.tap('AfterEmitPlugin', () => {
          const content = fs.readFileSync(
            path.resolve(testPath, 'bundle0.js'),
            'utf-8',
          );
          expect(content).toContain('sriExtractCssHashes');
          expect(content).toContain('"styles": "sha384-');
        });
      },
    }),
  ],
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'javascript/auto',
        use: [CssExtractRspackPlugin.loader, 'css-loader'],
      },
    ],
  },
}));
