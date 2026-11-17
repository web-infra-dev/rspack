import fs from 'node:fs';
import path from 'node:path';
import { defineConfig } from '@rspack/cli';
import type { Compiler } from '@rspack/core';

const removed = new Set<string>();

export default defineConfig({
  cache: { type: 'persistent', portable: { root: '.' } },
  module: { rules: [{ test: /value\.js$/, use: ['./loader.cjs'] }] },
  plugins: [
    (compiler: Compiler) => {
      compiler.hooks.afterCompile.tap('RemoveContext', () => {
        if (removed.has(compiler.context)) return;
        removed.add(compiler.context);
        // The loader has read the directory by now, and the snapshot has not been taken yet.
        fs.rmSync(path.join(compiler.context, 'dir'), { recursive: true });
      });
    },
  ],
});
