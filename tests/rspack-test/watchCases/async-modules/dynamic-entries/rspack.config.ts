import fs from 'node:fs';
import path from 'node:path';
import { defineConfig, definePlugin } from '@rspack/cli';
import type { Compiler } from '@rspack/core';

let compiler: Compiler;

export default defineConfig({
  entry: async () => {
    const context = compiler.context;
    const files = await fs.promises.readdir(context);
    let entries = files.filter((f) => f.startsWith('index'));
    entries.sort();
    return entries.reduce<Record<string, string>>((acc, e, i) => {
      acc[`bundle${i}`] = path.resolve(context, e);
      return acc;
    }, {});
  },
  output: {
    filename: '[name].js',
  },
  plugins: [
    definePlugin(function (c) {
      compiler = c;
    }),
  ],
});
