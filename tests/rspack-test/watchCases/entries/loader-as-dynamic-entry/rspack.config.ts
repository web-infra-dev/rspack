import { defineConfig, definePlugin } from '@rspack/cli';
import fs from 'node:fs';
import path from 'node:path';
/*
Ensures proper cleanup of dependency subtree when 404 entry is removed.
Without proper cleanup, this would throw a "Module not found" error for './404.js'.

× Module not found: Can't resolve './404.js'
.         ╭────
.       1 │ require("./404.js")
.         · ───────────────────
.         ╰────
*/

let MAIN: string;

let _404: string;

export default defineConfig({
  entry() {
    const entries: Record<string, string> = {};
    if (fs.existsSync(MAIN)) {
      entries['main'] = './main.js';
    }
    if (fs.existsSync(_404)) {
      entries['404'] = './404-page-loader.mjs!';
    }
    return entries;
  },
  output: {
    filename: '[name].js',
  },
  plugins: [
    definePlugin({
      apply(compiler) {
        MAIN = path.join(compiler.context, 'main.js');
        _404 = path.join(compiler.context, '404.js');

        compiler.hooks.finishMake.tap('PLUGIN', (compilation) => {
          if (!fs.existsSync(MAIN)) {
            compilation.missingDependencies.add(MAIN);
          }
          if (!fs.existsSync(_404)) {
            compilation.missingDependencies.add(_404);
          }
        });
      },
    }),
  ],
});
