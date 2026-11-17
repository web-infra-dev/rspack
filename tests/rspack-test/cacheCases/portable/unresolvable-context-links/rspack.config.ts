import fs from 'node:fs';
import path from 'node:path';
import { defineConfig } from '@rspack/cli';
import type { Compiler } from '@rspack/core';

export default defineConfig({
  cache: { type: 'persistent', portable: { root: '.' } },
  plugins: [
    (compiler: Compiler) => {
      compiler.hooks.beforeRun.tapPromise('ContextLinks', async () => {
        const directory = path.join(compiler.context, 'pages');
        if (fs.readdirSync(directory).includes('dangling')) return;
        fs.symlinkSync('missing', path.join(directory, 'dangling'));
        fs.symlinkSync('loop', path.join(directory, 'loop'));
        // Let the first-use timestamp guard settle before compilation reads the context.
        await new Promise((resolve) => setTimeout(resolve, 1100));
      });
    },
  ],
});
