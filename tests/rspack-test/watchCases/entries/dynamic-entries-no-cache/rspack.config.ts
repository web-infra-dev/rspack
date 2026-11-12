import { defineConfig, definePlugin } from '@rspack/cli';
import { type Compiler, rspack } from '@rspack/core';
import path from 'node:path';
import fs from 'node:fs';

let compiler: Compiler;
let step = 0;
let entries: string[];

export default defineConfig({
  entry: async () => {
    const context = compiler.context;
    if (step === 0) {
      const files = await fs.promises.readdir(context);
      entries = files.filter((f) => f.startsWith('index'));
      entries.sort();
    } else if (step === 1) {
    } else {
      throw new Error(`unreachable step: ${step}`);
    }
    return entries.reduce<Record<string, string>>((acc, e, i) => {
      acc[`bundle${i}`] = path.resolve(context, e);
      return acc;
    }, {});
  },
  output: {
    filename: '[name].js',
  },
  cache: false,
  plugins: [
    new rspack.experiments.RemoveDuplicateModulesPlugin(),
    definePlugin(function (c) {
      compiler = c;
      c.hooks.done.tap('test', () => {
        step += 1;
      });
    }),
  ],
});
