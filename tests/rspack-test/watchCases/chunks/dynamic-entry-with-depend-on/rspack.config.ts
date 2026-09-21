import fs from 'node:fs';
import path from 'node:path';
import { defineConfig, definePlugin } from '@rspack/cli';
import type { EntryObject } from '@rspack/core';

let MAIN: string;

export default defineConfig({
  entry(): EntryObject {
    if (fs.existsSync(MAIN)) {
      return {
        shared: './shared.js',
        main: {
          import: './main.js',
          dependOn: 'shared',
        },
      };
    }
    return {
      shared: './shared.js',
    };
  },
  output: {
    filename: '[name].js',
  },
  plugins: [
    definePlugin({
      apply(compiler) {
        MAIN = path.join(compiler.context, 'main.js');

        compiler.hooks.finishMake.tap('PLUGIN', (compilation) => {
          if (!fs.existsSync(MAIN)) {
            compilation.missingDependencies.add(MAIN);
          }
        });
      },
    }),
  ],
});
