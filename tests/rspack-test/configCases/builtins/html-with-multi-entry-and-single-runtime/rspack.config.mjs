import { rspack } from '@rspack/core';
import path from 'node:path';
import fs from 'node:fs';

/** @type {import("@rspack/core").Configuration} */
export default {
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
    {
      apply(compiler) {
        compiler.hooks.done.tap('TestAssert', () => {
          let outputPath = compiler.options.output.path;
          const htmlContent = fs.readFileSync(
            path.join(outputPath, 'index.html'),
            'utf-8',
          );

          expect(htmlContent.match(/runtime\.js/g)).toHaveLength(1);
        });
      },
    },
  ],
};
