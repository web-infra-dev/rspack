import { defineConfig } from '../../../../dist/index.js';
import type { Compiler } from '@rspack/core';
import path from 'node:path';

let count = 0;
const config = defineConfig({
  entry: path.resolve(import.meta.dirname, './index.js'),
  plugins: [
    {
      apply(compiler: Compiler) {
        compiler.hooks.afterCompile.tap('child', (base) => {
          if (count > 1) {
            return;
          }
          const child = base.createChildCompiler('child', {}, []);
          child.runAsChild(() => {
            count += 1;
          });
        });
      },
    },
  ],
});

export default config;
