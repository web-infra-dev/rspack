import fs from 'node:fs';
import path from 'node:path';
import { defineConfig, definePlugin } from '@rspack/cli';

const cacheDirectory = path.join(import.meta.dirname, '.cache');
let compilerIndex = 0;

export default defineConfig({
  mode: 'development',
  cache: {
    type: 'persistent',
    storage: { type: 'filesystem', directory: cacheDirectory },
  },
  plugins: [
    definePlugin((compiler) => {
      const index = compilerIndex++;
      compiler.options.bail = index !== 3;
      // Discard the bootstrap build's cache so the first failure is a cold build.
      if (index === 1) {
        fs.rmSync(cacheDirectory, { recursive: true, force: true });
      }
    }),
  ],
});
