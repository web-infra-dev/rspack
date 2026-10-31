import { defineConfig } from '../../../dist/index.js';
import type { Compiler } from '@rspack/core';
import path from 'node:path';

export default defineConfig({
  mode: 'development', // will be override to "production" by "--mode"
  extends: ['./base.config.mts'],
  cache: {
    type: 'persistent',
  },
  plugins: [
    {
      apply(compiler: Compiler) {
        const cache = compiler.options.cache;
        if (!cache || cache.type !== 'persistent') {
          return;
        }
        const [dep1, dep2] = cache.buildDependencies;
        if (
          dep1 === path.resolve(import.meta.dirname, './rspack.config.mts') &&
          dep2 === path.resolve(import.meta.dirname, './base.config.mts')
        ) {
          console.log('===buildDependencies===');
        }
      },
    },
  ],
});
