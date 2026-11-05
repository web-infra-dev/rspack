import { defineConfig } from '@rspack/cli';
import type { Compiler } from '@rspack/core';
import { rspack } from '@rspack/core';

export default defineConfig({
  context: import.meta.dirname,
  mode: 'development',
  entry: './src/index.js',
  stats: 'none',
  plugins: [
    new rspack.HtmlRspackPlugin({
      template: './src/index.html',
    }),
    {
      apply(compiler: Compiler) {
        compiler.hooks.compilation.tap('TestPlugin', (compilation) => {
          rspack.HtmlRspackPlugin.getCompilationHooks(
            compilation,
          ).alterAssetTags.tap('TestPlugin', (data) => {
            data.assetTags.scripts.push({
              tagName: 'script',
              innerHTML: 'console.log("injected source code");',
              voidTag: false,
              attributes: { id: 'inner-html-tag' },
            });
            return data;
          });
        });
      },
    },
  ],
  devServer: {
    port: 3000,
  },
});
