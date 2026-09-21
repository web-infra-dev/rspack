import type { Compiler } from '@rspack/core';
import { defineConfig } from '@rspack/cli';
import assert from 'node:assert';
import fs from 'node:fs';

export default defineConfig({
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: /\.png$/,
        type: 'asset/resource',
      },
    ],
  },
  plugins: [
    new (class {
      apply(compiler: Compiler) {
        compiler.hooks.compilation.tap('MyPlugin', (compilation) => {
          compilation.hooks.processAssets.tap('MyPlugin', (assets) => {
            let list = Object.keys(assets);
            const png = list.find((item) => item.endsWith('png'));
            assert(png);
            const asset = compilation.getAsset(png);
            assert(asset);
            const buf = asset.source.buffer();
            const expected = fs.readFileSync(
              import.meta.dirname + '/' + 'img.png',
            );
            assert.deepEqual(buf, expected);
          });
        });
      }
    })(),
  ],
});
