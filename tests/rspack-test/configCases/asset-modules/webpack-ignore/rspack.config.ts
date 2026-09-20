import { defineConfig, definePlugin } from '@rspack/cli';

import path from 'node:path';
import fs from 'node:fs';
import { rspack } from '@rspack/core';

const {
  Compilation,
  sources: { RawSource },
} = rspack;

export default defineConfig({
  mode: 'development',
  devtool: false,
  output: {
    module: true,
  },
  target: 'web',
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.compilation.tap('Test', (compilation) => {
          compilation.hooks.processAssets.tap(
            {
              name: 'TestCopyPlugin',
              stage: Compilation.PROCESS_ASSETS_STAGE_ADDITIONAL,
            },
            () => {
              const files = [
                'file.text',
                'file.json',
                'file.js',
                'file.css',
                'file.html',
              ];

              for (const file of files) {
                const testFile = path.resolve(import.meta.dirname, file);
                const content = fs.readFileSync(testFile);

                compilation.emitAsset(file, new RawSource(content));
              }
            },
          );
        });
      },
    }),
  ],
});
