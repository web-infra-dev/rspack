import { defineConfig } from '../../../dist/index.js';
import type { Compiler } from '@rspack/core';

export default defineConfig({
  mode: 'development',
  devtool: false,
  plugins: [
    {
      name: 'test-plugin',
      apply(compiler: Compiler) {
        compiler.hooks.compilation.tap('compilation', (compilation) => {
          compilation.hooks.processAssets.tap('assets', (assets) => {
            console.log({
              assets,
            });
          });
        });
      },
    },
  ],
  devServer: {},
});
