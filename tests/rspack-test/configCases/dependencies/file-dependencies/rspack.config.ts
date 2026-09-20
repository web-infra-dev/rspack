import { defineConfig, definePlugin } from '@rspack/cli';

import path from 'node:path';

export default defineConfig({
  plugins: [
    definePlugin((compiler) => {
      compiler.hooks.done.tap('Test', ({ compilation }) => {
        const fileDeps = Array.from(compilation.fileDependencies);
        expect(fileDeps).toContain(
          path.resolve(import.meta.dirname, 'node_modules/package/index.js'),
        );
        expect(fileDeps).toContain(
          path.resolve(import.meta.dirname, 'node_modules/package/extra.js'),
        );
        expect(fileDeps).toContain(
          path.resolve(import.meta.dirname, 'extra.js'),
        );
        expect(fileDeps).toContain(
          path.resolve(import.meta.dirname, 'index.js'),
        );
      });
    }),
  ],
});
