import type { Compiler } from '@rspack/core';
import { defineConfig } from '@rspack/cli';
import assert from 'node:assert';

export default defineConfig({
  context: import.meta.dirname,
  entry: {
    index: './img.png',
  },
  module: {
    rules: [
      {
        test: /\.png$/,
        type: 'asset/resource',
        generator: {
          importMode: 'preserve',
        },
      },
    ],
  },
  plugins: [
    new (class {
      apply(compiler: Compiler) {
        compiler.hooks.compilation.tap('MyPlugin', (compilation) => {
          compilation.hooks.processAssets.tap('MyPlugin', (assets) => {
            let list = Object.keys(assets);
            const js = list.find((item) => item.endsWith('js'));
            assert(js);
            const jsContent = assets[js].source().toString();
            assert(/require\(['"]\.\/(\w*)\.png['"]\)/.test(jsContent));
          });
        });
      }
    })(),
  ],
});
