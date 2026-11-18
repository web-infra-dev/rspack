import path from 'node:path';
import { defineConfig, definePlugin } from '@rspack/cli';
import { CssExtractRspackPlugin } from '@rspack/core';

let buildIndex = 0;

export default defineConfig({
  mode: 'development',
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'javascript/auto',
        use: [
          CssExtractRspackPlugin.loader,
          path.join(import.meta.dirname, 'css-tuple-loader.mjs'),
        ],
      },
    ],
  },
  plugins: [
    new CssExtractRspackPlugin(),
    definePlugin((compiler) => {
      compiler.hooks.done.tap('CssDependencySwitchTest', (stats) => {
        expect(stats.hasErrors()).toBe(false);
        const css = stats.compilation.getAsset('main.css').source.source();
        const color = buildIndex++ < 2 ? 'red' : 'blue';
        expect(String(css).trim()).toBe(`.test { color: ${color}; }`);
      });
    }),
  ],
});
