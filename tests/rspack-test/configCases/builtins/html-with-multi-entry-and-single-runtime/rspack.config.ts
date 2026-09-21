import assert from 'node:assert/strict';
import { defineConfig, definePlugin } from '@rspack/cli';
import { rspack } from '@rspack/core';
import path from 'node:path';
import fs from 'node:fs';

export default defineConfig({
  entry: {
    index: './index.js',
    other: './other-entry.js',
  },
  output: {
    filename: '[name].js',
  },
  optimization: {
    runtimeChunk: 'single',
  },
  plugins: [
    new rspack.HtmlRspackPlugin({
      template: './index.html',
    }),
    definePlugin({
      apply(compiler) {
        compiler.hooks.done.tap('TestAssert', () => {
          let outputPath = compiler.options.output.path;
          assert(outputPath);
          const htmlContent = fs.readFileSync(
            path.join(outputPath, 'index.html'),
            'utf-8',
          );

          expect(htmlContent.match(/runtime\.js/g)).toHaveLength(1);
        });
      },
    }),
  ],
});
