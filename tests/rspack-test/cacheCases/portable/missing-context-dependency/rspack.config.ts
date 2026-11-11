import { defineConfig } from '@rspack/cli';
import type { Compiler } from '@rspack/core';

const settled = new Set<string>();

export default defineConfig({
  cache: { type: 'persistent', portable: { root: '.' } },
  module: { rules: [{ test: /value\.js$/, use: ['./loader.cjs'] }] },
  plugins: [
    (compiler: Compiler) => {
      compiler.hooks.beforeRun.tapPromise('Settle', async () => {
        if (settled.has(compiler.context)) return;
        settled.add(compiler.context);
        // The fixture was just written. Let the first-use timestamp guard settle before the build.
        await new Promise((resolve) => setTimeout(resolve, 1100));
      });
    },
  ],
});
